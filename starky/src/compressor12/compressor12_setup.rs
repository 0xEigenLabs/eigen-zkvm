#![allow(non_snake_case)]
use crate::compressor12::plonk_setup::PlonkSetup;
use crate::errors::EigenError;
use algebraic::reader::load_r1cs;
use plonky::field_gl::GL;

pub type Result<T> = std::result::Result<T, EigenError>;

pub struct Options {
    pub force_bits: usize,
}

// setup phase:
// input: .r1cs
// output: .pil, .const, .exec,
pub fn setup(r1cs_file: &str, const_file: &str, force_n_bits: usize) -> Result<PlonkSetup> {
    // 0. readR1cs
    let r1cs = load_r1cs::<GL>(r1cs_file);
    let opts = Options {
        force_bits: force_n_bits,
    };

    // 1. plonk setup: generate plonk circuit, the pil_json.
    let res = PlonkSetup::new(&r1cs, &opts);

    // 2. write const pols file
    res.const_pols.save(const_file)?;

    Ok(res)
}
