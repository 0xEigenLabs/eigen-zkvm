#![allow(non_snake_case, dead_code)]
use crate::field_bn128::Fr;
use crate::helper::fr_to_biguint;
use core::slice;
use ff::*;
use std::fmt::Display;
use winter_math::StarkField;
use winter_math::{fields::f64::BaseElement, FieldElement};

use num_bigint::BigUint;
use num_traits::Num;
use num_traits::ToPrimitive;

const DIGEST_SIZE: usize = 4;

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ElementDigest(pub [BaseElement; DIGEST_SIZE]);

impl ElementDigest {
    pub fn new(value: [BaseElement; DIGEST_SIZE]) -> Self {
        Self(value)
    }

    pub fn as_elements(&self) -> &[BaseElement] {
        &self.0
    }

    pub fn _digests_as_elements(digests: &[Self]) -> &[BaseElement] {
        let p = digests.as_ptr();
        let len = digests.len() * DIGEST_SIZE;
        unsafe { slice::from_raw_parts(p as *const BaseElement, len) }
    }
}

impl Display for ElementDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}n\n{}n\n{}n\n{}n",
            self.0[0].as_int(),
            self.0[1].as_int(),
            self.0[2].as_int(),
            self.0[3].as_int()
        )
    }
}

/// Field mapping
/// Fr always consists of [u64; limbs], here for bn128, the limbs is 4.
impl From<&Fr> for ElementDigest {
    fn from(e: &Fr) -> Self {
        let mut result = [BaseElement::ZERO; DIGEST_SIZE];
        result[0] = BaseElement::from(e.0 .0[0]);
        result[1] = BaseElement::from(e.0 .0[1]);
        result[2] = BaseElement::from(e.0 .0[2]);
        result[3] = BaseElement::from(e.0 .0[3]);
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
    pub fn to_BN128(e: &[BaseElement; 4]) -> Fr {
        let mut result = BigUint::from(e[0].as_int());

        let mut added = BigUint::from(e[1].as_int());
        added = added << 64;
        result += added;

        let mut added = BigUint::from(e[2].as_int());
        added = added << 128;
        result += added;

        let mut added = BigUint::from(e[3].as_int());
        added = added << 192;
        result += added;

        Fr::from_str(&result.to_string()).unwrap()
    }

    fn to_GL(f: &Fr) -> [BaseElement; 4] {
        let mut f = fr_to_biguint(f);

        let mask = BigUint::from_str_radix("ffffffffffffffff", 16).unwrap();

        let mut result = [BaseElement::ZERO; 4];

        for i in 0..4 {
            let t = &f & &mask;
            result[i] = BaseElement::from(t.to_u64().unwrap());
            f = &f >> 64;
        }

        result
    }
}

impl Default for ElementDigest {
    fn default() -> Self {
        ElementDigest([BaseElement::default(); DIGEST_SIZE])
    }
}

impl From<[BaseElement; DIGEST_SIZE]> for ElementDigest {
    fn from(value: [BaseElement; DIGEST_SIZE]) -> Self {
        Self(value)
    }
}

impl From<ElementDigest> for [BaseElement; DIGEST_SIZE] {
    fn from(value: ElementDigest) -> Self {
        value.0
    }
}

#[cfg(test)]
pub mod tests {
    use crate::digest_bn128::ElementDigest;
    use crate::field_bn128::Fr;
    use ff::PrimeField;
    use rand_utils::rand_vector;
    use winter_math::fields::f64::BaseElement;

    #[test]
    fn test_fr_to_element_digest_and_versus() {
        let b4 = rand_vector::<BaseElement>(4);
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
        let b4: Vec<BaseElement> = vec![3u32, 1003, 2003, 0]
            .iter()
            .map(|e| BaseElement::from(e.clone()))
            .collect();
        let f1: Fr = ElementDigest::to_BN128(&b4[..].try_into().unwrap());

        // to Montgomery
        let f1 = Fr::from_repr(f1.into_raw_repr()).unwrap();

        let e1 = ElementDigest::to_GL(&f1);
        let expected: [BaseElement; 4] = vec![
            10593660675180540444u64,
            2538813791642109216,
            4942736554053463004,
            3183287946373923876,
        ]
        .iter()
        .map(|e| BaseElement::from(e.clone()))
        .collect::<Vec<BaseElement>>()
        .try_into()
        .unwrap();
        assert_eq!(expected, e1);
    }
}
