use fnv::FnvHasher;
use num::ToPrimitive;
use num_bigint::BigInt;
use num_traits::{One, Zero};
use serde_json::Value;
use std::hash::Hasher;
use std::str::FromStr;

pub(crate) fn fnv(inp: &str) -> (u32, u32) {
    let mut hasher = FnvHasher::default();
    hasher.write(inp.as_bytes());
    let h = hasher.finish();

    ((h >> 32) as u32, h as u32)
}

pub fn from_array32(arr: Vec<u32>) -> BigInt {
    let mut res = BigInt::zero();
    let radix = BigInt::from(0x100000000u64);
    for &val in arr.iter() {
        res = res * &radix + BigInt::from(val);
    }
    res
}

pub fn to_array32(s: &BigInt, size: usize) -> Vec<u32> {
    let mut res = vec![0; size];
    let mut rem = s.clone();
    let radix = BigInt::from(0x100000000u64);
    let mut c = size;
    while !rem.is_zero() {
        c -= 1;
        res[c] = (&rem % &radix).to_u32().unwrap();
        rem /= &radix;
    }

    res
}

#[allow(dead_code)]
pub fn value_to_bigint(v: Value) -> BigInt {
    match v {
        Value::String(inner) => BigInt::from_str(&inner).unwrap(),
        Value::Number(inner) => BigInt::from(inner.as_u64().expect("not a u32")),
        _ => panic!("unsupported type {:?}", v),
    }
}

pub fn flat_array(v: &[Value]) -> Vec<BigInt> {
    let mut result = Vec::new();
    fn fill_array(out: &mut Vec<BigInt>, value: &Value) {
        match value {
            Value::Array(inner) => {
                for v2 in inner.iter() {
                    fill_array(out, v2);
                }
            }
            Value::Bool(inner) => {
                if *inner {
                    out.push(BigInt::one());
                } else {
                    out.push(BigInt::zero());
                }
            }
            Value::String(inner) => {
                out.push(BigInt::from_str(inner).unwrap());
            }
            Value::Number(inner) => {
                out.push(BigInt::from_str(&inner.to_string()).unwrap());
            }
            _ => panic!(),
        }
    }

    for v2 in v.iter() {
        fill_array(&mut result, v2);
    }
    result
}
