use crate::compressor12_pil::CompressorNameSpace::Compressor;
use crate::compressor12_pil::CompressorPolName;
use crate::compressor12_pil::CompressorPolName::a;
use crate::errors::EigenError;
use crate::io_utils::read_vec_from_file;
use crate::pilcom::compile_pil_from_path;
use crate::polsarray::{Pol, PolKind, PolsArray};
use crate::r1cs2plonk::PlonkAdd;
use number::BigInt;
use plonky::api::calculate_witness;
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
    input_file: &String,
    wasm_file: &String,
    pil_file: &String,
    exec_file: &String,
    commit_file: &String,
) -> Result<()> {
    // 0. load exec_file,
    let (adds_len, s_map_column_len, adds, s_map) = read_exec_file(exec_file);

    // 1. Compiles a .pil file to its json form , and save it.
    // TODO: the pil_str has been compiled in plonk_setup#3
    let pil_json = compile_pil_from_path(pil_file);
    let mut file = File::create(Path::new(&format!("{pil_file}.json"))).unwrap();
    let input = serde_json::to_string(&pil_json).unwrap();
    write!(file, "{}", input);

    // 2. construct cmPol: .pil.json -> .cm
    let mut cm_pols = PolsArray::new(&pil_json, PolKind::Commit);

    // 3. calculate witness. wasm+input->witness
    let mut wtns = WitnessCalculator::new(wasm_file).unwrap();
    let inputs = load_input_for_witness(input_file);
    let w = wtns.calculate_witness(inputs, false)?;

    for i in 0..adds_len {
        // todo
        // let wi = w[adds[i * 4]] * adds[i * 4 + 2] + w[adds[i * 4 + 1]] * adds[i * 4 + 3];
        // w.push(wi);
    }

    // 4. compress cmPol
    // todo get value.
    // let N =
    for c in 0..12 {
        for i in 0..s_map_column_len {
            let s = s_map[c][i];
            let value = w[s];
            cm_pols.set_matrix(
                &Compressor.to_string(),
                &a.to_string(),
                c,
                i,
                if s != 0 {
                    // w[sMapBuff[12*i+j]]
                    FGL::ONE
                } else {
                    FGL::ZERO
                },
            );
        }
    }
    for c in 0..12 {
        // for i in 0..N {
        for i in 0..s_map_column_len {
            cm_pols.set_matrix(&Compressor.to_string(), &a.to_string(), c, i, FGL::ZERO);
        }
    }

    // 5. save cmPol to file.
    let mut file = File::create(Path::new(commit_file)).unwrap();
    let input = serde_json::to_string_pretty(&cm_pols).unwrap();
    write!(file, "{}", input);

    log::info!("files Generated Correctly");
    Result::Ok(())
}

fn read_exec_file(exec_file: &String) -> (usize, usize, Vec<PlonkAdd>, Vec<Vec<u64>>) {
    let buff = read_vec_from_file(exec_file).unwrap();

    let adds_len = buff[0] as usize;
    let s_map_column_len = buff[1] as usize;

    let size = 2 + adds_len * 4 + s_map_column_len * 12;
    assert_eq!(buff.len(), size);

    let mut adds = vec![];

    // todo check
    for i in 0..adds_len {
        let addi = PlonkAdd(
            buff[2 + i * 4] as usize,
            buff[2 + i * 4 + 1] as usize,
            FGL::from(buff[2 + i * 4 + 2]),
            FGL::from(buff[2 + i * 4 + 3]),
        );
        adds.push(addi);
    }

    let mut s_map = vec![vec![0; s_map_column_len]; 12];
    for c in 0..12 {
        for i in 0..s_map_column_len {
            s_map[c][i] = buff[2 + adds_len * 4 + 12 * i + c];
        }
    }

    (adds_len, s_map_column_len, adds, s_map)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compressor12_setup::write_exec_file;
    use std::io::Read;

    #[test]
    fn test_write_and_read_exec_file() {
        let file_path = String::from("./test_write_and_read_exec_file.txt");

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

        let (adds_len, s_map_column_len, adds, s_map) = read_exec_file(&file_path);

        assert_eq!(adds_len, target_adds.len());
        // assert_eq!(adds, target_adds);

        assert_eq!(12, s_map.len());
        assert_eq!(s_map_column_len, s_map[0].len());
        assert_eq!(target_s_map, s_map);
    }
}
