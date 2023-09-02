use crate::pilcom::{compile_pil, BackendType};
use plonky::api::calculate_witness;
use std::path::Path;

// exec phase:
// input files: .wasm, .exec,  .pil, zkin.json(input file),
// output: .cm
pub fn exec(
    wasm_file: &String,
    exec_file: &String,
    pil_file: &String,
    input_file: &String,
    commit_file: &String,
) -> Result<Ok(), Err()> {
    // 1. .pil -> cm
    //    Compiles a .pil file to its json form
    //    and generate constants and committed polynomials to file.(under the output_file_dir)
    let output_file_dir = Path::new(commit_file).parent()?;
    let _ = compile_pil(
        Path::new(pil_file),
        &output_file_dir,
        None,
        Some(BackendType::PilcomCli),
    );

    // // 2. wasm -> wc
    // const wc = await WitnessCalculatorBuilder(wasm);
    // // 3. input + wc -> w
    // const w = await wc.calculateWitness(input);
    calculate_witness(wasm_file, input_file, commit_file);

    //
    // for (let i=0; i<nAdds; i++) {
    //     w.push( F.add( F.mul( w[addsBuff[i*4]], addsBuff[i*4 + 2]), F.mul( w[addsBuff[i*4+1]],  addsBuff[i*4+3]  )));
    // }
    //
    // 4. w + cm + exec.addition -> final cm

    Result::Ok(())
}
