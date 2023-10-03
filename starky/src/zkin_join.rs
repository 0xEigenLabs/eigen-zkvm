use crate::errors::Result;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

/// Combine the `input1.zkin.json` and `input1.zkin.json` into one(`out.zkin.json`)
// ../../target/release/eigen-zkit join_zkin --zkin1 0/fibonacci.recursive1/input.zkin.json  --zkin2 1/fibonacci.recursive1/input.zkin.json  --zkinout 0/fibonacci.recursive1/r1_input-rs.zkin.json
pub fn join_zkin(
    // stark_setup_file: &String,
    zkin1: &String,
    zkin2: &String,
    zkout: &String,
) -> Result<()> {
    // 1. load files.
    // porting from compressor12_exec.(input_file)
    // let stark_struct = load_json::<StarkStruct>(&stark_setup_file).unwrap();
    let inputs_str = std::fs::read_to_string(zkin1).unwrap();
    let zkin1_map: BTreeMap<String, serde_json::Value> = serde_json::from_str(&inputs_str)?;

    let inputs_str = std::fs::read_to_string(zkin2).unwrap();
    let zkin2_map: BTreeMap<String, serde_json::Value> = serde_json::from_str(&inputs_str)?;

    // 2. construct zkout
    // node /Users/paul/blockchain/eigen-zkvm/test/../starkjs/src/recursive/main_joinzkin.js
    //      --starksetup ../starky/data/c12.starkStruct.json
    //      --zkin1 /tmp/aggregation_bn128_fibonacci/aggregation/0/fibonacci.recursive1/input.zkin.json
    //      --zkin2 /tmp/aggregation_bn128_fibonacci/aggregation/1/fibonacci.recursive1/input.zkin.json
    //      --zkinout /tmp/aggregation_bn128_fibonacci/aggregation/0/fibonacci.recursive1/r1_input.zkin.json
    let mut zkout_map = BTreeMap::new();

    for (k, v) in zkin1_map {
        zkout_map.insert(format!("a_{k}"), v);
    }
    for (k, v) in zkin2_map {
        zkout_map.insert(format!("b_{k}"), v);
    }

    // 3. save zkout to file
    // dump zkin file porting from stark_prove
    let input = serde_json::to_string(&zkout_map)?;
    let mut file = File::create(&zkout)?;
    write!(file, "{}", input).unwrap();
    log::info!("zkout file Generated Correctly");
    Ok(())
}
