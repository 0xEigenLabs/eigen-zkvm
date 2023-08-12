#![allow(non_snake_case, dead_code)]
use crate::field_bn128::{Fr, FrRepr};
use crate::helper::fr_to_biguint;
use core::slice;
use ff::*;
use plonky::field_gl::Fr as FGL;
use std::fmt::Display;

use num_bigint::BigUint;
use num_traits::Num;
use num_traits::ToPrimitive;

const DIGEST_SIZE: usize = 4;

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ElementDigest(pub [FGL; DIGEST_SIZE]);

impl ElementDigest {
    pub fn new(value: [FGL; DIGEST_SIZE]) -> Self {
        Self(value)
    }

    pub fn as_elements(&self) -> &[FGL] {
        &self.0
    }

    pub fn _digests_as_elements(digests: &[Self]) -> &[FGL] {
        let p = digests.as_ptr();
        let len = digests.len() * DIGEST_SIZE;
        unsafe { slice::from_raw_parts(p as *const FGL, len) }
    }
}

impl Display for ElementDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}",
            self.0[0].as_int(),
            self.0[1].as_int(),
            self.0[2].as_int(),
            self.0[3].as_int()
        )
    }
}

/// Fr always consists of [u64; limbs], here for bn128, the limbs is 4.
impl From<&Fr> for ElementDigest {
    fn from(e: &Fr) -> Self {
        let mut result = [FGL::ZERO; DIGEST_SIZE];
        result[0] = FGL::from(e.0 .0[0]);
        result[1] = FGL::from(e.0 .0[1]);
        result[2] = FGL::from(e.0 .0[2]);
        result[3] = FGL::from(e.0 .0[3]);
        ElementDigest::new(result)
    }
}

impl Into<Fr> for ElementDigest {
    fn into(self) -> Fr {
        let mut result = Fr::zero();
        result.0 .0[0] = self.0[0].as_int() as u64;
        result.0 .0[1] = self.0[1].as_int() as u64;
        result.0 .0[2] = self.0[2].as_int() as u64;
        result.0 .0[3] = self.0[3].as_int() as u64;
        result
    }
}

impl ElementDigest {
    #[inline(always)]
    pub fn to_bn128(e: &[FGL; 4]) -> Fr {
        let mut buf: Vec<u8> = vec![0u8; 32];
        // To be optimized: FGL doesn't return bytes with specific endian.
        buf[0..8].copy_from_slice(&e[0].as_int().to_le_bytes());
        buf[8..16].copy_from_slice(&e[1].as_int().to_le_bytes());
        buf[16..24].copy_from_slice(&e[2].as_int().to_le_bytes());
        buf[24..32].copy_from_slice(&e[3].as_int().to_le_bytes());
        let mut repr = FrRepr::default();
        let required_length = repr.as_ref().len() * 8;
        buf.resize(required_length, 0);
        repr.read_le(&buf[..]).unwrap();
        Fr::from_repr(repr).unwrap()
    }

    // for debug only
    fn to_gl(f: &Fr) -> [FGL; 4] {
        let mut f = fr_to_biguint(f);
        let mask = BigUint::from_str_radix("ffffffffffffffff", 16).unwrap();
        let mut result = [FGL::ZERO; 4];
        for i in 0..4 {
            let t = &f & &mask;
            result[i] = FGL::from(t.to_u64().unwrap());
            f = &f >> 64;
        }
        result
    }
}

impl Default for ElementDigest {
    fn default() -> Self {
        ElementDigest([FGL::ZERO; DIGEST_SIZE])
    }
}

impl From<[FGL; DIGEST_SIZE]> for ElementDigest {
    fn from(value: [FGL; DIGEST_SIZE]) -> Self {
        Self(value)
    }
}

impl From<ElementDigest> for [FGL; DIGEST_SIZE] {
    fn from(value: ElementDigest) -> Self {
        value.0
    }
}

#[cfg(test)]
pub mod tests {
    use crate::digest::ElementDigest;
    use crate::field_bn128::Fr;
    use ff::{Field, PrimeField};
    use plonky::field_gl::Fr as FGL;
    use rand::Rand;

    #[test]
    fn test_fr_to_element_digest_and_versus() {
        let mut rng = ::rand::thread_rng();
        let b4 = vec![
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
        ];
        let b4 = ElementDigest::new(b4.try_into().unwrap());
        let f1: Fr = b4.into();

        let b4_: ElementDigest = ElementDigest::from(&f1);
        assert_eq!(b4, b4_);

        let f: Fr = Fr::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495616", // Fr::MODULE - 1
        )
        .unwrap();

        let e = ElementDigest::from(&f);
        let f2: Fr = e.into();
        assert_eq!(f, f2);
    }

    #[test]
    fn test_fr_to_mont_to_element_digest_and_versus() {
        let b4: Vec<FGL> = vec![3u64, 1003, 2003, 0]
            .iter()
            .map(|e| FGL::from(e.clone()))
            .collect();
        let f1: Fr = ElementDigest::to_bn128(&b4[..].try_into().unwrap());

        // to Montgomery
        let f1 = Fr::from_repr(f1.into_raw_repr()).unwrap();

        let e1 = ElementDigest::to_gl(&f1);
        let expected: [FGL; 4] = vec![
            10593660675180540444u64,
            2538813791642109216,
            4942736554053463004,
            3183287946373923876,
        ]
        .iter()
        .map(|e| FGL::from(e.clone()))
        .collect::<Vec<FGL>>()
        .try_into()
        .unwrap();
        assert_eq!(expected, e1);
    }
}
