#![allow(non_snake_case)]

use crate::compressor12_exec::exec;
use crate::compressor12_setup::setup;
use algebraic::errors::EigenError;
pub type Result<T> = std::result::Result<T, EigenError>;

pub mod compressor12_exec;
pub(crate) mod compressor12_pil;
pub mod compressor12_setup;
pub(crate) mod constants;
pub(crate) mod plonk_setup;

// compress12 phase:
// input: .r1cs, .wasm, zkin.json(input_file)
// output: .const, .cm, pil.json
pub fn compress12(
    force_n_bits: usize,
    r1cs_file: &str,
    wasm_file: &str,
    input_file: &str,
    const_file: &str,
    commit_file: &str,
    pil_json_file: &str,
) -> Result<()> {
    // setup phase:
    // input: .r1cs
    // output: .pil, .const, .exec,
    let plonk_setup = setup(r1cs_file, const_file, force_n_bits)?;

    // exec phase:
    // input files: .wasm, .exec,  .pil, zkin.json(input file),
    // output: .cm, .pil.json
    exec(
        plonk_setup,
        input_file,
        wasm_file,
        pil_json_file,
        commit_file,
    )
}
