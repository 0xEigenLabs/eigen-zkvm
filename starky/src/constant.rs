#![allow(non_snake_case)]
use crate::poseidon_bn128::Fr;
use ff::*;

use std::collections::HashMap;
use winter_math::{fft, fields::f64::BaseElement};

lazy_static::lazy_static! {
    pub static ref OFFSET_2_64: Fr = Fr::from_str("18446744073709551616").unwrap();
    pub static ref OFFSET_2_128: Fr = Fr::from_str("340282366920938463463374607431768211456").unwrap();
    pub static ref OFFSET_2_192: Fr = Fr::from_str("6277101735386680763835789423207666416102355444464034512896").unwrap();
    pub static ref CHALLENGE_MAP: HashMap<&'static str, i32> = {
        let mut m = HashMap::new();
        m.insert("u", 0);
        m.insert("defVal", 1);
        m.insert("gamma", 2);
        m.insert("beta", 3);
        m.insert("vc", 4);
        m.insert("vf1", 5);
        m.insert("vf2", 6);
        m.insert("xi", 7);
        m
    };

    pub static ref SHIFT: BaseElement = BaseElement::from(49u32);
    pub static ref TWIDDLES: Vec<BaseElement> = fft::get_twiddles::<BaseElement>(2usize.pow(32));
}

pub const MIN_OPS_PER_THREAD: usize = 1 << 12;
pub const MAX_OPS_PER_THREAD: usize = 1 << 16;

pub fn get_max_workers() -> usize {
    num_cpus::get() - 1
}
