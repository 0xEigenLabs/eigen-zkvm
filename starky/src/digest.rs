#![allow(non_snake_case)]
use crate::field_bn128::{Fr, FrRepr};
use crate::traits::MTNodeType;
use ff::*;
use plonky::field_gl::Fr as FGL;
use std::fmt::Display;

// bn254
// const DIGEST_SIZE: usize = 4;
// bls12-381
// const BLS12381_DIGEST_SIZE: usize = 6;

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ElementDigest<const N: usize>(pub [FGL; N]);

impl<const N: usize> MTNodeType for ElementDigest<N> {
    #[inline(always)]
    fn new(value: &[FGL]) -> Self {
        assert_eq!(value.len() >= N, true);
        let mut fv = [FGL::ZERO; N];
        for i in 0..N {
            fv[i] = value[i];
        }
        Self(fv)
    }

    #[inline(always)]
    fn as_elements(&self) -> &[FGL] {
        &self.0
    }

    #[inline(always)]
    fn from_scalar<T: PrimeField>(e: &T) -> Self {
        let mut result = [FGL::ZERO; N];
        let ee = e.into_raw_repr();
        let eee = ee.as_ref();
        for i in 0..N {
            result[i] = FGL::from(eee[i]);
        }
        ElementDigest::new(&result)
    }

    // TODO generic implement
    #[inline(always)]
    fn as_bn128(self) -> Fr {
        let mut result = Fr::zero();
        for i in 0..N {
            result.0 .0[i] = self.0[i].as_int();
        }
        result
    }
}

impl<const N: usize> Display for ElementDigest<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..N {
            write!(f, "{}\n", self.0[i].as_int())?;
        }
        Ok(())
    }
}

impl<const N: usize> Default for ElementDigest<N> {
    #[inline(always)]
    fn default() -> Self {
        ElementDigest::<N>([FGL::ZERO; N])
    }
}

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

#[cfg(test)]
mod tests {
    use crate::digest::to_bn128;
    use crate::digest::ElementDigest;
    use crate::field_bn128::Fr;
    use crate::helper::fr_to_biguint;
    use crate::traits::MTNodeType;
    use ff::PrimeField;
    use num_bigint::BigUint;
    use num_traits::Num;
    use num_traits::ToPrimitive;
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
        let b4 = ElementDigest::new(&b4);
        let f1: Fr = b4.as_bn128();

        let b4_: ElementDigest<4> = ElementDigest::from_scalar(&f1);
        assert_eq!(b4, b4_);

        let f: Fr = Fr::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495616", // Fr::MODULE - 1
        )
        .unwrap();

        let e = ElementDigest::<4>::from_scalar(&f);
        let f2: Fr = e.as_bn128();
        assert_eq!(f, f2);
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

    #[test]
    fn test_fr_to_mont_to_element_digest_and_versus() {
        let b4: Vec<FGL> = vec![3u64, 1003, 2003, 0]
            .iter()
            .map(|e| FGL::from(e.clone()))
            .collect();
        let f1: Fr = to_bn128(&b4[..].try_into().unwrap());

        // to Montgomery
        let f1 = Fr::from_repr(f1.into_raw_repr()).unwrap();

        let e1 = to_gl(&f1);
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
