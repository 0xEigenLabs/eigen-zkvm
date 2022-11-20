use crate::poseidon_bn128::Fr;
use ff::*;
use std::ops::Mul;

///exports.getKs = function getKs(Fr, n) {
///    const ks = [Fr.k];
///    for (let i=1; i<n; i++) {
///        ks[i] = Fr.mul(ks[i-1], ks[0]);
///    }
///    return ks;
///}
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;

use num_bigint::BigUint;
use num_traits::Num;
use num_traits::ToPrimitive;

pub fn get_ks(n: usize) -> Vec<BaseElement> {
    let mut ks: Vec<BaseElement> = vec![BaseElement::ZERO; n];
    ks[0] = BaseElement::from(12275445934081160404u64);
    for i in 1..n {
        ks[i] = ks[i - 1].mul(ks[0])
    }
    ks
}

pub fn fr_to_biguint(f: &Fr) -> BigUint {
    let se = to_hex(f);
    let se = se.trim_end_matches('0');
    match se.len() {
        0 => BigUint::from(0u32),
        _ => BigUint::from_str_radix(&se, 16).unwrap(),
    }
}

pub fn biguint_to_be(f: &BigUint) -> BaseElement {
    let module = BigUint::from(0xFFFFFFFF00000001u64);
    let f = f % module;
    BaseElement::from(f.to_u64().unwrap())
}

pub fn biguint_to_fr(f: &BigUint) -> Fr {
    Fr::from_str(&f.to_string()).unwrap()
}
