use crate::compressor12::plonk_setup::PlonkSetup;
use crate::compressor12_pil::CompressorNameSpace::*;
use crate::compressor12_pil::CompressorPolName::a;
use crate::errors::EigenError;
use crate::polsarray::{PolKind, PolsArray};
use num_traits::Zero;
use plonky::ff::PrimeField;
use plonky::field_gl::Fr as FGL;
use plonky::witness::{load_input_for_witness, WitnessCalculator};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub type Result<T> = std::result::Result<T, EigenError>;

// exec phase:
// input files: .wasm, .exec,  .pil, zkin.json(input file),
// output: .cm
pub fn exec(
    plonk_setup: PlonkSetup,
    input_file: &str,
    wasm_file: &str,
    pil_json_file: &str,
    commit_file: &str,
) -> Result<()> {
    // 0. prepare data,
    let adds = plonk_setup.plonk_additions;
    let s_map = plonk_setup.s_map;
    let adds_len = adds.len();
    let s_map_column_len = s_map[0].len();

    // 1. save pil_json data..
    let pil_json = plonk_setup.pil_json;
    let mut file = File::create(Path::new(pil_json_file)).unwrap();
    let input = serde_json::to_string(&pil_json).unwrap();
    write!(file, "{}", input).unwrap();

    // 2. construct cmPol: .pil.json -> .cm
    let mut cm_pols = PolsArray::new(&pil_json, PolKind::Commit);

    // 3. calculate witness. wasm+input->witness
    let mut wtns = WitnessCalculator::new(wasm_file).unwrap();
    let inputs = load_input_for_witness(input_file);
    let w = wtns.calculate_witness(inputs, false).unwrap();
    let mut w = w
        .iter()
        .map(|wi| {
            if wi.is_zero() {
                FGL::ZERO
            } else {
                assert!(wi.to_u64_digits().1.len() < 2);
                FGL::from(wi.to_u64_digits().1[0])
            }
        })
        .collect::<Vec<_>>();

    for i in 0..adds_len {
        // TODO: here we can's assign `let w2 = adds[i].2;`. As adds[i].2 is mont form. But here w2 need mont_reduce form.?
        let a2: u64 = adds[i].2.into();
        let a3: u64 = adds[i].3.into();
        let w2 = FGL::from_raw_repr(<FGL as PrimeField>::Repr::from(a2))?;
        let w3 = FGL::from_raw_repr(<FGL as PrimeField>::Repr::from(a3))?;

        let f_w = (w[adds[i].0] * w2) + (w[adds[i].1] * w3);
        w.push(f_w);
    }

    // 4. compress cmPol
    let a_np_index = cm_pols.get_pol_id(&pil_json, &Compressor.to_string(), &a.to_string(), 0);
    let N = cm_pols.array[a_np_index].len();

    for i in 0..s_map_column_len {
        for c in 0..12 {
            let s = s_map[c][i] as usize;

            cm_pols.set_matrix(
                &pil_json,
                &Compressor.to_string(),
                &a.to_string(),
                c,
                i,
                if s != 0 { w[s] } else { FGL::ZERO },
            );
        }
    }
    for i in s_map_column_len..N {
        for c in 0..12 {
            cm_pols.set_matrix(
                &pil_json,
                &Compressor.to_string(),
                &a.to_string(),
                c,
                i,
                FGL::ZERO,
            );
        }
    }

    // 5. save cmPol to file.
    cm_pols.save(commit_file)?;

    log::debug!("files Generated Correctly");
    Ok(())
}
