#![allow(non_snake_case)]
use anyhow::Result;
use crate::field_bls12381::Fr as Fr_bls12381;
use crate::field_bls12381::FrRepr as FrRepr_bls12381;
use crate::field_bn128::{Fr, FrRepr};
use crate::traits::MTNodeType;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use ff::*;
use fields::field_gl::Fr as FGL;
use std::fmt::Display;
use std::io::{Read, Write};

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ElementDigest<const N: usize>(pub [FGL; N]);

impl<const N: usize> MTNodeType for ElementDigest<N> {
    #[inline(always)]
    fn new(value: &[FGL]) -> Self {
        assert!(value.len() >= N);
        let mut fv = [FGL::ZERO; N];
        fv[..N].copy_from_slice(&value[..N]);
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

    /// This function may return a invalid T due to it's just a container for T's inner elements.
    #[inline(always)]
    fn as_scalar<T: PrimeField>(&self) -> T::Repr {
        let mut y = T::Repr::default();
        let t = y.as_mut();
        for (i, ti) in t.iter_mut().enumerate().take(N) {
            *ti = self.0[i].as_int();
        }
        y
    }

    fn save<W: Write>(&self, writer: &mut W) -> Result<()> {
        for i in &self.0 {
            writer.write_u64::<LittleEndian>(i.as_int())?;
        }
        Ok(())
    }

    fn load<R: Read>(reader: &mut R) -> Result<Self> {
        let e1 = reader.read_u64::<LittleEndian>()?;
        let e2 = reader.read_u64::<LittleEndian>()?;
        let e3 = reader.read_u64::<LittleEndian>()?;
        let e4 = reader.read_u64::<LittleEndian>()?;
        Ok(Self::new(&[e1.into(), e2.into(), e3.into(), e4.into()]))
    }
}

impl<const N: usize> Display for ElementDigest<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..N {
            writeln!(f, "{}", self.0[i].as_int())?;
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

#[inline(always)]
pub fn to_bls12381(e: &[FGL; 4]) -> Fr_bls12381 {
    let mut buf: Vec<u8> = vec![0u8; 32];
    // To be optimized: FGL doesn't return bytes with specific endian.
    buf[0..8].copy_from_slice(&e[0].as_int().to_le_bytes());
    buf[8..16].copy_from_slice(&e[1].as_int().to_le_bytes());
    buf[16..24].copy_from_slice(&e[2].as_int().to_le_bytes());
    buf[24..32].copy_from_slice(&e[3].as_int().to_le_bytes());
    let mut repr = FrRepr_bls12381::default();
    let required_length = repr.as_ref().len() * 8;
    buf.resize(required_length, 0);
    repr.read_le(&buf[..]).unwrap();
    Fr_bls12381::from_repr(repr).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::digest::to_bls12381;
    use crate::digest::to_bn128;
    use crate::digest::ElementDigest;
    use crate::field_bls12381::Fr as Fr_bls12381;
    use crate::field_bn128::Fr;
    use crate::helper::{fr_bls12381_to_biguint, fr_to_biguint};
    use crate::traits::MTNodeType;
    use ff::PrimeField;
    use num_bigint::BigUint;
    use num_traits::Num;
    use num_traits::ToPrimitive;
    use fields::field_gl::Fr as FGL;
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
        let f1: Fr = Fr(b4.as_scalar::<Fr>());

        let b4_: ElementDigest<4> = ElementDigest::from_scalar(&f1);
        assert_eq!(b4, b4_);

        let f: Fr = Fr::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495616", // Fr::MODULE - 1
        )
        .unwrap();

        let e = ElementDigest::<4>::from_scalar(&f);
        let f2: Fr = Fr(e.as_scalar::<Fr>());
        assert_eq!(f, f2);
    }

    // for debug only
    fn to_gl(f: &Fr) -> [FGL; 4] {
        let mut f = fr_to_biguint(f);
        let mask = BigUint::from_str_radix("ffffffffffffffff", 16).unwrap();
        let mut result = [FGL::ZERO; 4];
        for res in &mut result {
            let t = &f & &mask;
            *res = FGL::from(t.to_u64().unwrap());
            f = &f >> 64;
        }
        result
    }

    // for debug only
    fn to_gl_bls12381(f: &Fr_bls12381) -> [FGL; 4] {
        let mut f = fr_bls12381_to_biguint(f);
        let mask = BigUint::from_str_radix("ffffffffffffffff", 16).unwrap();
        let mut result = [FGL::ZERO; 4];
        for res in &mut result {
            let t = &f & &mask;
            *res = FGL::from(t.to_u64().unwrap());
            f = &f >> 64;
        }
        result
    }

    #[test]
    fn test_fr_to_mont_to_element_digest_and_versus() {
        let b4: Vec<FGL> = [3u64, 1003, 2003, 0]
            .iter()
            .map(|e| FGL::from(*e))
            .collect();
        let f1: Fr = to_bn128(&b4[..].try_into().unwrap());

        // to Montgomery
        let f1 = Fr::from_repr(f1.into_raw_repr()).unwrap();

        let e1 = to_gl(&f1);
        let expected: [FGL; 4] = [
            10593660675180540444u64,
            2538813791642109216,
            4942736554053463004,
            3183287946373923876,
        ]
        .iter()
        .map(|e| FGL::from(*e))
        .collect::<Vec<FGL>>()
        .try_into()
        .unwrap();
        assert_eq!(expected, e1);
    }

    #[test]
    fn test_fr_bls12381_to_mont_to_element_digest_and_versus() {
        let b4: Vec<FGL> = [3u64, 1003, 2003, 0]
            .iter()
            .map(|e| FGL::from(*e))
            .collect();
        let f1: Fr_bls12381 = to_bls12381(&b4[..].try_into().unwrap());

        // to Montgomery
        let f1 = Fr_bls12381::from_repr(f1.into_raw_repr()).unwrap();

        let e1 = to_gl_bls12381(&f1);
        let expected: [FGL; 4] = [
            11023535560112151624u64,
            10252228934103205545,
            1509485146568764231,
            1588734141810477816,
        ]
        .iter()
        .map(|e| FGL::from(*e))
        .collect::<Vec<FGL>>()
        .try_into()
        .unwrap();
        assert_eq!(expected, e1);
    }
}
