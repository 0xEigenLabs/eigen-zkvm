mod circom;
pub(crate) mod memory;
pub(super) mod witness_calculator;

use fnv::FnvHasher;
use num_bigint::BigInt;
use num_traits::{One, Zero};
use serde_json::Value;
use std::collections::HashMap;
use std::hash::Hasher;
use std::str::FromStr;
pub use witness_calculator::flat_array;
pub use witness_calculator::WitnessCalculator;

pub(crate) fn fnv(inp: &str) -> (u32, u32) {
    let mut hasher = FnvHasher::default();
    hasher.write(inp.as_bytes());
    let h = hasher.finish();

    ((h >> 32) as u32, h as u32)
}

pub fn load_input_for_witness(input_file: &str) -> HashMap<String, Vec<BigInt>> {
    let inputs_str = std::fs::read_to_string(input_file).unwrap_or_else(|_|  panic!("Unable to load {}", input_file));
    let inputs: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&inputs_str).unwrap();

    inputs
        .iter()
        .map(|(key, value)| {
            let res = match value {
                Value::String(inner) => {
                    vec![BigInt::from_str(inner).unwrap()]
                }
                Value::Bool(inner) => {
                    if *inner {
                        vec![BigInt::one()]
                    } else {
                        vec![BigInt::zero()]
                    }
                }
                Value::Number(inner) => {
                    vec![BigInt::from_str(&inner.to_string()).unwrap()]
                    //vec![BigInt::from(inner.as_u64().expect("not a u32"))]
                }
                //Value::Array(inner) => inner.iter().cloned().map(value_to_bigint).collect(),
                Value::Array(inner) => flat_array(inner),
                _ => panic!("{:?}", value),
            };

            (key.clone(), res)
        })
        .collect::<std::collections::HashMap<_, _>>()
}
