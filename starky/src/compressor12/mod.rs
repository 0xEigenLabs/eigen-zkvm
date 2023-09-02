mod compressor12_exec;
pub mod compressor12_pil;
pub mod compressor12_setup;

use crate::compressor12_setup::{plonk_setup_render, Options};
use crate::pilcom::{compile_pil, BackendType};
use crate::types::{load_json, PIL};
use plonky::api::calculate_witness;
use plonky::field_gl::GL;
use plonky::reader::load_r1cs;
use std::path::Path;

// todo async
// the inputs here include r1cs and it's input, c12 pil. And output the const file and exec file (this means that comments in the script are not correct), the process basically likes
//
// The r1cs is the constraints of the Stark Verifier, and will be converted to Plonk gate;
// Generate the Pil code for the Plonk gate;
// Generate the const polynomial and commit polynomial for the Pil code. but here it does not output all the commit directly, cause it still need the c12a.pil to contrain somes computation, like poseidon, fft etc.
//
//
// generate the pil files,  const polynomials files, the commit files
//  input files :  $C12_VERIFIER.r1cs
//  output files :  $C12_VERIFIER.const, $C12_VERIFIER.pil,  $C12_VERIFIER.cm
//
// NOTE: Compare the raw one, here we skip the .exec file, produce the .const and .cm file together.
// todo: How to deal with the input file?
#[deprecated]
pub fn setup(
    r1cs_file: &String,
    const_file: &String,
    pil_file: &String,
    force_n_bits: usize,
) -> Result<Ok(), Err()> {
    let opts = Options {
        force_bits: force_n_bits,
    };

    // 0. load r1cs
    let r1cs = load_r1cs::<GL>(r1cs_file);
    // 1. generate plonk circuit, the pil file.
    //      And write it into pil_file.
    let (plonk_setup_info, pil_str) = plonk_setup_render(&r1cs, &opts, pil_file);

    // 2. Compiles a .pil file to its json form
    // 3/4. and generate constants and committed polynomials to file.(under the output_file_dir)
    let output_file_dir = Path::new(const_file).parent()?;
    let _ = compile_pil(
        Path::new(pil_file),
        &output_file_dir,
        None,
        Some(BackendType::PilcomCli),
    );

    Result::Ok(())
}
