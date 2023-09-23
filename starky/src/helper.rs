#![allow(dead_code)]
use crate::field_bls12381::Fr as Fr_bls12381;
use crate::field_bn128::Fr;
use ff::*;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use plonky::field_gl::Fr as FGL;
use plonky::Field;
use std::fmt::Write;
use std::ops::Mul;

///exports.getKs = function getKs(Fr, n) {
///    const ks = [Fr.k];
///    for (let i=1; i<n; i++) {
///        ks[i] = Fr.mul(ks[i-1], ks[0]);
///    }
///    return ks;
///}
pub fn get_ks(n: usize) -> Vec<FGL> {
    let mut ks: Vec<FGL> = vec![FGL::ZERO; n];
    ks[0] = FGL::from(12275445934081160404u64);
    for i in 1..n {
        ks[i] = ks[i - 1].mul(ks[0])
    }
    ks
}

#[inline(always)]
pub fn log2_any(val: usize) -> usize {
    let mut val = val;
    (if (val & 0xFFFF0000) != 0 {
        val &= 0xFFFF0000;
        16
    } else {
        0
    }) | (if (val & 0xFF00FF00) != 0 {
        val &= 0xFF00FF00;
        8
    } else {
        0
    }) | (if (val & 0xF0F0F0F0) != 0 {
        val &= 0xF0F0F0F0;
        4
    } else {
        0
    }) | (if (val & 0xCCCCCCCC) != 0 {
        val &= 0xCCCCCCCC;
        2
    } else {
        0
    }) | (if (val & 0xAAAAAAAA) != 0 { 1 } else { 0 })
}

#[inline(always)]
pub fn fr_to_biguint(f: &Fr) -> BigUint {
    let repr = f.into_repr();
    let required_length = repr.as_ref().len() * 8;
    let mut buf: Vec<u8> = Vec::with_capacity(required_length);
    repr.write_be(&mut buf).unwrap();
    BigUint::from_bytes_be(&buf)
}

#[inline(always)]
pub fn fr_bls12381_to_biguint(f: &Fr_bls12381) -> BigUint {
    let repr = f.into_repr();
    let required_length = repr.as_ref().len() * 8;
    let mut buf: Vec<u8> = Vec::with_capacity(required_length);
    repr.write_be(&mut buf).unwrap();
    BigUint::from_bytes_be(&buf)
}

#[inline(always)]
pub fn biguint_to_be(f: &BigUint) -> FGL {
    let module = BigUint::from(0xFFFFFFFF00000001u64);
    let f = f % module;
    FGL::from(f.to_u64().unwrap())
}

#[inline(always)]
pub fn biguint_to_fr(f: &BigUint) -> Fr {
    Fr::from_str(&f.to_string()).unwrap()
}

#[inline(always)]
pub fn biguint_bls12381_to_fr(f: &BigUint) -> Fr_bls12381 {
    Fr_bls12381::from_str(&f.to_string()).unwrap()
}

pub fn pretty_print_array<T: Field>(cols: &Vec<T>) -> String {
    let mut msg = String::new();
    writeln!(&mut msg, "array size: {}", cols.len()).unwrap();
    writeln!(&mut msg, "[").unwrap();
    let mut iglines = 2;
    for i in 0..8 {
        if cols.len() > i {
            writeln!(&mut msg, "  {}", cols[i]).unwrap();
            iglines += 1;
        }
    }
    if iglines < cols.len() {
        writeln!(&mut msg, "  ...{}s...", cols.len() - iglines).unwrap();
        writeln!(&mut msg, "  {}", cols[cols.len() - 1]).unwrap();
    }
    writeln!(&mut msg, "]").unwrap();
    msg
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
