#![allow(non_snake_case)]
use crate::poseidon_bn128::Fr;
use ff::*;

use crate::f3g::F3G;
use std::collections::HashMap;
use winter_math::{fft, fields::f64::BaseElement, FieldElement};

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

    pub static ref SHIFT: F3G = F3G::from(BaseElement::from(49u32));
    pub static ref SHIFT_INV: F3G = F3G::inv(SHIFT.clone());
    pub static ref W: (Vec<F3G>, Vec<F3G>) = {
        let nqr = F3G::from(BaseElement::from(7u32));
        let rem = 2usize.pow(32) - 1;
        let s = 32usize;
        let mut w = vec![F3G::ZERO; s+1];
        let mut wi = vec![F3G::ZERO; s+1];
        w[s] = nqr.pow(rem);
        wi[s] = w[s].inv();

        for n in (0..s).rev() {
            w[n] = w[n+1].square();
            wi[n] = wi[n+1].square();
        }
        (w, wi)
    };
}

pub const MIN_OPS_PER_THREAD: usize = 1 << 12;
pub const MAX_OPS_PER_THREAD: usize = 1 << 16;

pub fn get_max_workers() -> usize {
    num_cpus::get() - 1
}
