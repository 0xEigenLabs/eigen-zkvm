#![allow(unused_imports)]

use crate::ff::PrimeField;
use core::ops::{Add, Div, Mul, Neg, Sub};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, PrimeField)]
#[PrimeFieldModulus = "18446744069414584321"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
pub struct Goldilocks([u64; 2]);

impl Goldilocks {
    pub const fn new(first: u64, second: u64) -> Self {
        Goldilocks([first, second])
    }
    pub const fn get(&self) -> u64 {
        self.0[0]
    }
}

#[cfg(test)]
mod tests {
    use super::Goldilocks;
    use ff::{Field, PrimeField};
    use proptest::prelude::*;
    use std::ops::Neg;

    proptest! {
        #[test]
        fn gl_check_add(a in any::<u64>()) {
            let v = Goldilocks::from_str_vartime(&a.to_string()).unwrap();
            let added = v + v;
            let double = v * Goldilocks::from_str_vartime("2").unwrap();
            prop_assert_eq!(added, double);
        }

        #[test]
        fn gl_check_sub(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Goldilocks::from_str_vartime(&a.to_string()).unwrap();
            let v2 = Goldilocks::from_str_vartime(&b.to_string()).unwrap();
            let lhs = v2 - v1;
            let rhs = lhs + v1;
            prop_assert_eq!(v2, rhs);
        }

        #[test]
        fn gl_check_mul(a in any::<u64>()) {
            let v = Goldilocks::from_str_vartime(&a.to_string()).unwrap();
            let lhs = v * v * v;
            let rhs = v.square();
            prop_assert_eq!(lhs, rhs * v);
        }

        #[test]
        fn gl_check_inv(a in any::<u64>()) {
            let v = Goldilocks::from_str_vartime(&a.to_string()).unwrap();
            let v_inversed = v.invert().unwrap();
            prop_assert_eq!(v * v_inversed, Goldilocks::ONE);
        }


        #[test]
        fn gl_check_neg(a in any::<u64>()){
            let v1 = Goldilocks::from_str_vartime(&a.to_string()).unwrap();
            let v2 = v1.neg();
            prop_assert_eq!(v1+v2, Goldilocks::ZERO);
        }

        #[test]
        fn gl_check_sqrt(a in any::<u64>()) {
            let v = Goldilocks::from_str_vartime(&a.to_string()).unwrap();
            match v.sqrt() {
                ct_option if ct_option.is_some().into() => {
                    let value = ct_option.unwrap();
                    let squared = value.square();
                    prop_assert_eq!(squared, v);
                },
                _ => {}
            }
        }
    }

    #[test]
    fn test_serde_and_deserde() {
        let data = Goldilocks::ONE;
        let serialized = serde_json::to_string(&data).unwrap();
        println!("Serialized: {}", serialized);

        let expect: Goldilocks = serde_json::from_str(&serialized).unwrap();

        assert_eq!(data, expect);
    }
}
