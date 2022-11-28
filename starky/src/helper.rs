use crate::poseidon_bn128::Fr;
use ff::*;
use num_bigint::BigUint;
use num_traits::{Num, ToPrimitive};
use std::ops::Mul;
use winter_math::{fields::f64::BaseElement, FieldElement, StarkField};

///exports.getKs = function getKs(Fr, n) {
///    const ks = [Fr.k];
///    for (let i=1; i<n; i++) {
///        ks[i] = Fr.mul(ks[i-1], ks[0]);
///    }
///    return ks;
///}
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

pub fn pretty_print_matrix(cols: &Vec<Vec<BaseElement>>) {
    println!("matrix: cols {}, rows: {}", cols.len(), cols[0].len());
    for i in 0..cols.len() {
        println!("cols: {} begin", i);
        println!("\t cols[{}][0]: {}", i, cols[i][0].as_int());
        //println!("\t cols[{}][1]: {}", i, cols[i][1].as_int());
        //println!("\t cols[{}][2]: {}", i, cols[i][2].as_int());
        //println!("\t cols[...]: {} lines ignored", cols.len() - 5);
        //println!("\t cols[{}][2]: {}", i, cols[i][cols[i].len() - 2].as_int());
        println!(
            "\t cols[{}][row-1]: {}",
            i,
            cols[i][cols[i].len() - 1].as_int()
        );
        println!("cols: {} end", i);
    }
}
