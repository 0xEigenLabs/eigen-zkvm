#![allow(non_snake_case)]
use crate::poseidon_bn128::Fr;
use ff::*;

lazy_static::lazy_static! {
    pub static ref OFFSET_2_64: Fr = Fr::from_str("18446744073709551616").unwrap();
    pub static ref OFFSET_2_128: Fr = Fr::from_str("340282366920938463463374607431768211456").unwrap();
    pub static ref OFFSET_2_192: Fr = Fr::from_str("6277101735386680763835789423207666416102355444464034512896").unwrap();
}

pub const min_ops_per_thread: usize = 1 << 12;
pub const max_ops_per_thread: usize = 1 << 16;

pub fn get_max_workers() -> usize {
    num_cpus::get() - 1
}
