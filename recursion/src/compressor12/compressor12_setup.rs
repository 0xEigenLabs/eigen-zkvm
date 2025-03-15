#![allow(non_snake_case)]
use crate::compressor12::plonk_setup::PlonkSetup;
use crate::io_utils::write_vec_to_file;
use crate::r1cs2plonk::PlonkAdd;
use algebraic::reader::load_r1cs_from_bin;
use anyhow::{anyhow, Result};
use fields::field_gl::GL;
use std::fs::File;
use std::io::Write;

pub struct Options {
    pub force_bits: usize,
}

// setup phase:
// input: .r1cs
// output: .pil, .const, .exec,
pub fn setup(
    r1cs_file: &str,
    pil_file: &str,
    const_file: &str,
    exec_file: &str,
    force_n_bits: usize,
) -> Result<()> {
    // 0. readR1cs
    let r1cs_reader = File::open(r1cs_file)?;
    let (r1cs, _) = load_r1cs_from_bin::<_, GL>(r1cs_reader);
    let opts = Options { force_bits: force_n_bits };

    // 1. plonk setup: generate plonk circuit, the pil file.
    let res = PlonkSetup::new(&r1cs, &opts);

    // 2. And write it into pil_file.
    let mut file =
        File::create(pil_file).map_err(|_| anyhow!("Create file error, {}", pil_file))?;
    write!(file, "{}", res.pil_str)?;

    // 3. write const pols file
    let const_writer = std::fs::File::create(const_file)?;
    res.const_pols.save(const_writer)?;

    let mut file_writer =
        File::create(exec_file).map_err(|e| anyhow!("Create {}, {:?}", exec_file, e))?;
    // 4. construct and save ExecFile: plonk additions + sMap -> BigUint64Array
    write_exec_file(&mut file_writer, &res.plonk_additions, &res.s_map)?;

    Ok(())
}

// construct and save ExecFile: plonk additions + sMap -> BigUint64Array
pub(super) fn write_exec_file<W: std::io::Write>(
    exec_file_writer: &mut W,
    adds: &[PlonkAdd],
    s_map: &[Vec<u64>],
) -> Result<()> {
    let adds_len = adds.len();
    let s_map_row_len = s_map.len();
    let s_map_column_len = s_map[0].len();

    assert_eq!(s_map_row_len, 12, "s_map should have 12 rows");
    let size = 2 + adds_len * 4 + s_map_row_len * s_map_column_len;

    let mut buff = vec![0; size];

    buff[0] = adds_len as u64;
    buff[1] = s_map_column_len as u64;

    for i in 0..adds_len {
        buff[2 + i * 4] = adds[i].0 as u64;
        buff[2 + i * 4 + 1] = adds[i].1 as u64;
        buff[2 + i * 4 + 2] = adds[i].2.into();
        buff[2 + i * 4 + 3] = adds[i].3.into();
    }

    // TODO: Should this be a fixed constant or use the s_map_row_len.
    for c in 0..12 {
        for i in 0..s_map_column_len {
            buff[2 + adds_len * 4 + 12 * i + c] = s_map[c][i];
        }
    }

    write_vec_to_file(exec_file_writer, &buff)
}
