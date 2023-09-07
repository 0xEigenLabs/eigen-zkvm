use crate::pilcom::{compile_pil, BackendType};
use plonky::api::calculate_witness;
use std::path::Path;

// exec phase:
// input files: .wasm, .exec,  .pil, zkin.json(input file),
// output: .cm
pub fn exec(
    input_file: &String,
    wasm_file: &String,
    pil_file: &String,
    exec_file: &String,
    commit_file: &String,
) -> Result<Ok(), Err()> {
    // 0. load input_file, wasm_file, pil_file, exec_file,

    read_exec_file(execFile);

    // 1. Compiles a .pil file to its json form
    //      And save it.
    // todo-the pil file has been compiled in setup-plonk_setup phase.

    // 2. construct cmPol: .pil.json -> .cm
    // const cmPols = newCommitPolsArray(pil);

    // 3. calculate witness. wasm+input->witness
    // const wc = await WitnessCalculatorBuilder(wasm);
    // const w = await wc.calculateWitness(input);
    // calculate_witness(wasm_file, input_file, commit_file);

    // 4. compress cmPol

    // 5. save cmPol to file.

    Result::Ok(())
}

fn read_exec_file(exec_file: &String) {

    // const fd =await fs.promises.open(execFile, "r");
    // const buffH = new BigUint64Array(2);
    // await fd.read(buffH, 0, 2*8);
    // const nAdds= Number(buffH[0]);
    // const nSMap= Number(buffH[1]);
    //
    //
    // const addsBuff = new BigUint64Array(nAdds*4);
    // await fd.read(addsBuff, 0, nAdds*4*8);
    //
    // const sMapBuff = new BigUint64Array(nSMap*12);
    // await fd.read(sMapBuff, 0, nSMap*12*8);
    //
    // await fd.close();
    //
    // return { nAdds, nSMap, addsBuff, sMapBuff };
}
