use crate::compressor12_pil::CompressorNameSpace::*;
use crate::compressor12_pil::CompressorPolName::a;
use crate::errors::EigenError;
use crate::io_utils::read_vec_from_file;
use crate::pilcom::compile_pil_from_path;
use crate::polsarray::{PolKind, PolsArray};
use algebraic::r1cs_witness::witness_calculator::WitnessCalculator;
use num_traits::Zero;
use plonky::ff::PrimeField;
use plonky::field_gl::Fr as FGL;
use plonky::r1cs_witness::load_input_for_witness;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub type Result<T> = std::result::Result<T, EigenError>;

// exec phase:
// input files: .wasm, .exec,  .pil, zkin.json(input file),
// output: .cm
pub fn exec(
    input_file: &str,
    wasm_file: &str,
    pil_file: &str,
    exec_file: &str,
    commit_file: &str,
) -> Result<()> {
    // 0. load exec_file,
    let (adds_len, s_map_column_len, adds, s_map) = read_exec_file(exec_file);

    // 1. Compiles a .pil file to its json form , and save it.
    // TODO: the pil_str has been compiled in plonk_setup#3
    let pil_json = compile_pil_from_path(pil_file);
    let mut file = File::create(Path::new(&format!("{pil_file}.json"))).unwrap();
    let input = serde_json::to_string(&pil_json).unwrap();
    write!(file, "{}", input).unwrap();

    // 2. construct cmPol: .pil.json -> .cm
    let mut cm_pols = PolsArray::new(&pil_json, PolKind::Commit);

    // 3. calculate r1cs_witness. wasm+input->r1cs_witness
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
        let w2 = FGL::from_raw_repr(<FGL as PrimeField>::Repr::from(adds[i * 4 + 2])).unwrap();
        let w3 = FGL::from_raw_repr(<FGL as PrimeField>::Repr::from(adds[i * 4 + 3])).unwrap();

        let f_w = (w[adds[i * 4] as usize] * w2) + (w[adds[i * 4 + 1] as usize] * w3);
        w.push(f_w);
    }

    // 4. compress cmPol
    let a_np_index = cm_pols.get_pol_id(&pil_json, &Compressor.to_string(), &a.to_string(), 0);
    let N = cm_pols.array[a_np_index].len();

    for i in 0..s_map_column_len {
        for c in 0..12 {
            let s = s_map[i * 12 + c] as usize;

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
    Result::Ok(())
}

fn read_exec_file(exec_file: &str) -> (usize, usize, Vec<u64>, Vec<u64>) {
    let mut buff = read_vec_from_file(exec_file).unwrap();

    let mut new_buff = buff.split_off(2);
    let adds_len = buff[0] as usize;
    let s_map_column_len = buff[1] as usize;

    let size = adds_len * 4 + s_map_column_len * 12;
    assert_eq!(new_buff.len(), size);

    let s_map = new_buff.split_off(adds_len * 4);
    let adds = new_buff;

    (adds_len, s_map_column_len, adds, s_map)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compressor12_setup::write_exec_file;

    #[test]
    fn test_write_and_read_exec_file() {
        let file_path = String::from("/tmp/test_write_and_read_exec_file.txt");

        let target_adds = vec![
            // PlonkAdd()
        ];

        let target_s_map = vec![
            vec![1, 2, 4],
            vec![2, 3, 42],
            vec![1, 1, 3],
            vec![4, 5, 2],
            vec![3, 4, 5],
            vec![1, 2, 4],
            vec![2, 3, 42],
            vec![1, 1, 3],
            vec![4, 5, 2],
            vec![3, 4, 5],
            vec![3, 4, 5],
            vec![3, 4, 5],
        ];

        write_exec_file(&file_path, &target_adds, &target_s_map);

        let (adds_len, _s_map_column_len, _adds, _s_map) = read_exec_file(&file_path);

        assert_eq!(adds_len, target_adds.len());
    }
}
