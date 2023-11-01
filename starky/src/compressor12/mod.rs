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
// output: .const, .cm
pub fn compress12(
    // setup
    r1cs_file: &str,
    // pil_file: &str,
    const_file: &str,
    // exec_file: &str,
    force_n_bits: usize,
    // exec
    input_file: &str,
    wasm_file: &str,
    pil_file: &str,
    exec_file: &str,
    commit_file: &str,
) -> Result<()> {
    // todo remove the pil_file and exec_file.

    // setup phase:
    // input: .r1cs
    // output: .pil, .const, .exec,
    // return: todo PIL, exec file.
    let plonk_setup = setup(
        r1cs_file,
        // pil_file,
        const_file,
        // exec_file,
        force_n_bits,
    )?;

    // exec phase:
    // input files: .wasm, .exec,  .pil, zkin.json(input file),
    // output: .cm
    exec(
        plonk_setup,
        input_file,
        wasm_file,
        // pil_file, exec_file,
        commit_file,
    )?;
}
