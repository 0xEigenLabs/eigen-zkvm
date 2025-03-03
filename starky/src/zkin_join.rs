use anyhow::Result;
use serde_json::Value;
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
    let inputs_str = std::fs::read_to_string(zkin1)?;
    let zkin1_map: BTreeMap<String, serde_json::Value> = serde_json::from_str(&inputs_str)?;

    let inputs_str = std::fs::read_to_string(zkin2)?;
    let zkin2_map: BTreeMap<String, serde_json::Value> = serde_json::from_str(&inputs_str)?;

    // 2. construct zkout
    let mut zkout_map = BTreeMap::new();

    for (k, v) in zkin1_map {
        // TODO: remove this
        zkout_map.insert(format!("a_{k}"), v.clone());
        if k == "publics" {
            if let Value::Array(ref arr) = v {
                if arr.len() >= 4 {
                    let exclude_last_four = &arr[..(arr.len() - 4)];
                    zkout_map
                        .insert("publics".to_string(), Value::Array(exclude_last_four.to_vec()));
                } else {
                    zkout_map.insert("publics".to_string(), v.clone());
                }
            }
        }
        if k == "rootC" {
            zkout_map.insert(k.to_string(), v.clone());
        }
    }
    for (k, v) in zkin2_map {
        zkout_map.insert(format!("b_{k}"), v);
    }

    // 3. save zkout to file
    let input = serde_json::to_string(&zkout_map)?;
    let mut file = File::create(zkout)?;
    write!(file, "{}", input)?;
    log::trace!("zkout file Generated Correctly");
    Ok(())
}
