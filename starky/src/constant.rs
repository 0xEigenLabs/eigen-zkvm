#![allow(non_snake_case)]
use crate::f3g::F3G;
use crate::field_bn128::Fr;
use crate::poseidon_bn128::{load_constants, Constants};
use crate::poseidon_bn128_opt::load_constants as load_constants_opt;
use ff::*;
use plonky::field_gl::Fr as FGL;
use plonky::Field;
use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref OFFSET_2_64: Fr = Fr::from_str("18446744073709551616").unwrap();
    pub static ref OFFSET_2_128: Fr = Fr::from_str("340282366920938463463374607431768211456").unwrap();
    pub static ref OFFSET_2_192: Fr = Fr::from_str("6277101735386680763835789423207666416102355444464034512896").unwrap();
    pub static ref CHALLENGE_MAP: HashMap<&'static str, usize> = {
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

    pub static ref SHIFT: F3G = F3G::from(FGL::from(49u64));
    pub static ref SHIFT_INV: F3G = F3G::inv(SHIFT.clone());
    pub static ref MG: (Vec<F3G>, Vec<F3G>) = {
        let nqr = F3G::from(FGL::from(7u64));
        let rem = 2usize.pow(32) - 1;
        let s = 32usize;
        let mut w = vec![F3G::ZERO; s+1];
        let mut wi = vec![F3G::ZERO; s+1];
        w[s] = nqr.exp(rem);
        wi[s] = w[s].inv();

        for n in (0..s).rev() {
            let mut tmp = w[n+1].clone();
            tmp.square();
            w[n] =tmp;
            let mut tmp1 = wi[n+1].clone();
            tmp1.square();
            wi[n] = tmp1;
        }
        (w, wi)
    };

    pub static ref POSEIDON_BN128_CONSTANTS_OPT: Constants = {
        load_constants_opt()
    };
    pub static ref POSEIDON_BN128_CONSTANTS: Constants = {
        load_constants()
    };
    pub static ref POSEIDON_CONSTANTS_OPT: crate::poseidon_opt::Constants = {
        crate::poseidon_opt::load_constants()
    };
}

pub const MIN_OPS_PER_THREAD: usize = 1 << 12;
pub const MAX_OPS_PER_THREAD: usize = 1 << 18;

pub fn get_max_workers() -> usize {
    num_cpus::get() - 1
}
