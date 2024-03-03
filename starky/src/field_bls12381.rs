#![allow(unused_imports, clippy::too_many_arguments)]
use core::ops::{Add, Div, Mul, Neg, Sub};
use ff::*;
use serde::{Deserialize, Serialize};

#[derive(PrimeField, Serialize, Deserialize)]
#[PrimeFieldModulus = "52435875175126190479447740508185965837690552500527637822603658699938581184513"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(pub FrRepr);

#[cfg(test)]
mod tests {
    use crate::field_bls12381::*;
    use ff::*;
    use ff::{Field, PrimeField};
    use num_bigint::BigInt;
    use rand::Rand;
    use std::ops::{Add, Mul, Neg, Sub};
    #[test]
    fn test_ff_bls12381() {
        assert_eq!(
            format!("{:?}", Fr::zero()),
            "Fr(0x0000000000000000000000000000000000000000000000000000000000000000)"
        );
        assert_eq!(
            format!("{:?}", Fr::one()),
            "Fr(0x0000000000000000000000000000000000000000000000000000000000000001)"
        );
        assert_eq!(
            format!("{:?}", R),
            "0x1824b159acc5056f998c4fefecbc4ff55884b7fa0003480200000001fffffffe"
        );
    }

    #[test]
    fn test_ff_bls12381_equality() {
        assert_eq!(Fr::zero(), Fr::zero());
        assert_eq!(Fr::one(), Fr::one());
        assert_eq!(R2, R2);

        assert!(Fr::zero() != Fr::one());
        assert!(Fr::one() != Fr::from_repr(R2).unwrap());
    }

    #[test]
    fn test_ff_bls12381_add() {
        let mut f1 = Fr::from_repr(FrRepr([
            0xc81265fb4130fe0c,
            0xb308836c14e22279,
            0x699e887f96bff372,
            0x84ecc7e76c11ad,
        ]))
        .unwrap();
        let f2 = Fr::from_repr(FrRepr([
            0x71875719b422efb8,
            0x43658e68a93612,
            0x9fa756be2011e833,
            0xaa2b2cb08dac497,
        ]))
        .unwrap();
        let f3 = Fr::from_repr(FrRepr([
            0x3999bd14f553edc4,
            0xb34be8fa7d8b588c,
            0x945df3db6d1dba5,
            0xb279f92f046d645,
        ]))
        .unwrap();
        f1.add_assign(&f2);
        assert_eq!(f1, f3);
    }

    #[test]
    fn test_ff_bls12381_mul() {
        let mut rng = rand::thread_rng();
        let v = Fr::rand(&mut rng);
        let mut lhs = v;
        lhs.mul_assign(&v);
        lhs.mul_assign(&v);
        let mut rhs = v;
        rhs.square();
        rhs.mul_assign(&v);
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn test_ff_bls12381_from_str() {
        let f100_from_repr = Fr::from_repr(FrRepr([0x64, 0, 0, 0])).unwrap();
        let f100 = Fr::from_str("100").unwrap();
        assert_eq!(f100_from_repr, f100);
    }
}
