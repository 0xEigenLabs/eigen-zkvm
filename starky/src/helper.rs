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

pub fn log2_any(V: usize) -> usize {
    let mut V = V;
    (if (V & 0xFFFF0000) != 0 {
        V &= 0xFFFF0000;
        16
    } else {
        0
    }) | (if (V & 0xFF00FF00) != 0 {
        V &= 0xFF00FF00;
        8
    } else {
        0
    }) | (if (V & 0xF0F0F0F0) != 0 {
        V &= 0xF0F0F0F0;
        4
    } else {
        0
    }) | (if (V & 0xCCCCCCCC) != 0 {
        V &= 0xCCCCCCCC;
        2
    } else {
        0
    }) | (if (V & 0xAAAAAAAA) != 0 { 1 } else { 0 })
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

pub fn pretty_print_matrix<T: FieldElement + StarkField>(cols: &Vec<Vec<T>>) {
    if cols.len() == 0 {
        return;
    }
    println!("matrix: cols {}, rows: {}", cols.len(), cols[0].len());
    let mut iglines = 2;
    let width = 10;
    for i in 0..cols[0].len() {
        print!("\t rows[{:?}]: {:?},", i, cols[0][i].as_int());
        if cols.len() > 2 {
            print!("{:?},", cols[1][i].as_int());
            print!("{:?},", cols[2][i].as_int());
            iglines += 2;
        }
        if iglines < cols.len() {
            print!(" .{:?}s. ", cols.len() - iglines);
        }
        if cols.len() > 2 {
            print!("{:#?}", cols[cols[i].len() - 2][i].as_int());
        }
        println!("{:#?}.", cols[cols.len() - 1][i].as_int());
    }
}

pub fn pretty_print_array<T: FieldElement + StarkField>(cols: &Vec<T>) {
    println!("array size: {}", cols.len());
    let mut iglines = 2;
    print!("\t [ {:?},", cols[0].as_int());
    if cols.len() > 2 {
        print!("{:#?},", cols[1].as_int());
        print!("{:#?},", cols[2].as_int());
        iglines += 2;
    }
    if iglines < cols.len() {
        print!(" .{:?}s. ", cols.len() - iglines);
    }
    if cols.len() > 2 {
        print!("{:?}", cols[cols.len() - 2].as_int());
    }
    println!("{:?}].", cols[cols.len() - 1].as_int());
}

#[cfg(test)]
mod tests {
    use crate::helper::log2_any;

    // https://users.rust-lang.org/t/logarithm-of-integers/8506/4
    const fn num_bits<T>() -> usize {
        std::mem::size_of::<T>() * 8
    }
    fn log_2(x: usize) -> usize {
        assert!(x > 0);
        num_bits::<usize>() as usize - x.leading_zeros() as usize - 1
    }

    #[test]
    fn test_log2() {
        for i in 1..100 {
            assert_eq!(log2_any(i), log_2(i));
        }
    }
}
