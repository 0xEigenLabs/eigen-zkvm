use crate::errors::EigenError;
use crate::pilcom::compile_pil;
use crate::types::load_json;
use plonky::api::calculate_witness;
use std::fs::File;
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
    // 0. load input_file, wasm_file, pil_file, exec_file,

    // let (num_adds, num_s_map, adds_buff, s_map_buff) = read_exec_file(execFile);

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
    let mut buf = load_json(exec_file).unwrap();
    let mut file = File::open(exec_file)?;
    let mut data = String::new();
    file.read(&mut data)?;

    // buff[0] = adds_len;
    // buff[1] = s_map.len();
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
