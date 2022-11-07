#![allow(non_snake_case)]
use crate::poseidon_bn128::{Fr, Poseidon};
use crate::ElementDigest;
use ff::*;
use winter_crypto::{Digest, Hasher};
use winter_math::fields::f64::BaseElement;
use winter_math::{FieldElement, StarkField};

use crate::errors::Result;
use crate::traits::FieldMapping;

pub struct LinearHashBN128 {
    h: Poseidon,
}

use crate::constant::*;

impl LinearHashBN128 {
    pub fn new() -> Self {
        LinearHashBN128 { h: Poseidon::new() }
    }

    /// used for hash leaves only, converting element from GL to BN128
    /// columns:
    ///    0, 0, 0,
    ///    1, 1, 1,
    ///      ...,
    ///    n, n, n,
    pub fn hash_element_matrix(&self, columns: &Vec<Vec<BaseElement>>) -> Result<Fr> {
        let mut st = Fr::zero();
        let mut vals3: Vec<Fr> = vec![];

        let mut acc = Fr::zero();
        let mut accN = 0;

        for col in columns.iter() {
            for elem in col.iter() {
                // NOTE: BaseElement to Fr
                let mut e = Fr::from_str(&elem.as_int().to_string()).unwrap();
                if accN == 1 {
                    e.mul_assign(&OFFSET_2_64);
                } else if accN == 2 {
                    e.mul_assign(&OFFSET_2_128);
                }
                acc.add_assign(&e);
                accN += 1;
                if accN == 3 {
                    vals3.push(acc);
                    acc = Fr::zero();
                    accN = 0;
                }
            }
        }
        if accN > 0 {
            vals3.push(acc);
        }
        if vals3.len() == 0 {
            return Ok(st);
        } else if vals3.len() == 1 {
            return Ok(vals3[0]);
        }
        let mut inHash: Vec<Fr> = vec![];

        for val3 in vals3.iter() {
            inHash.push(val3.clone());
            if inHash.len() == 16 {
                st = self.h.hash(&inHash, &st)?;
                inHash = vec![];
            }
        }
        if inHash.len() > 0 {
            st = self.h.hash(&inHash, &st)?;
        }
        Ok(st)
    }

    /// convert to BN128 in montgomery
    pub fn to_bn128_mont(st64: &[BaseElement; 4]) -> [BaseElement; 4] {
        let bn: Fr = ElementDigest::to_BN128(st64);
        let bn_mont = ElementDigest::to_montgomery(&bn);
        ElementDigest::to_GL(&bn_mont)
    }

    pub fn inner_hash_block(&self, elems: &[BaseElement], init_state: &Fr) -> Result<Fr> {
        println!("inner_hash_block size: {}", elems.len());
        let elems = elems
            .chunks(4)
            .map(|e| {
                let r = ElementDigest::to_BN128(e.try_into().unwrap());
                /*let r = ElementDigest::to_montgomery(&bn);

                let ee = ElementDigest::to_GL(&r);
                ee
                    .iter()
                    .map(|e| {
                        print!(" {}", e.as_int())
                    })
                .collect::<Vec<()>>();
                */
                r
            })
            .collect::<Vec<Fr>>();
        println!("\nelem.length {:?}, {:?}", elems.len(), elems);
        Ok(self.h.hash(&elems, init_state)?)
    }

    /// columns:
    ///    0, 0, 0,  -> element
    ///    1, 1, 1,  -> element
    ///      ...,
    ///    n, n, n,
    pub fn hash_element_array(&self, vals: &Vec<BaseElement>) -> Result<ElementDigest> {
        let mut st64 = [BaseElement::ZERO; 4];
        let mut in64: [BaseElement; 64] = [BaseElement::ZERO; 64];
        let mut digest: Fr = Fr::zero();
        //println!("hash_element_array size: {}", vals.len());
        if vals.len() <= 4 {
            for (i, v) in vals.iter().enumerate() {
                st64[i] = *v;
            }
            let gl_mont = Self::to_bn128_mont(&st64);
            return Ok(ElementDigest::from(gl_mont));
        }

        let mut p = 0;

        for (i, val) in vals.iter().enumerate() {
            in64[p] = *val;
            p += 1;
            if p == 16 * 4 {
                digest = self.inner_hash_block(&in64[..], &digest)?;
                p = 0;
            }
            if i % 3 == 2 {
                in64[p] = BaseElement::ZERO;
                p += 1;
                if p == 16 * 4 {
                    digest = self.inner_hash_block(&in64[..], &digest)?;
                    p = 0;
                }
            }
        }
        if p > 0 {
            let nLast = (p - 1) / 4 + 1;
            while p < nLast * 4 {
                in64[p] = BaseElement::ZERO;
                p += 1;
            }
            digest = self.inner_hash_block(&in64[..(nLast * 4)], &digest)?;
        }
        Ok(ElementDigest::from(&digest))
    }
}

/// asher element over BN128
/*
impl Hasher for LinearHashBN128 {
    type Digest = ElementDigest;

    // implement instance.exports.poseidon(pSt, pIn, 16, pSt, 1);
    fn hash(bytes: &[u8]) -> Self::Digest {
        let hasher = Self::new();
        let elems: &[BaseElement] = unsafe { BaseElement::bytes_as_elements(bytes).unwrap() };
        //println!("Hasher::hash {:?}", elems);
        let digest = hasher.hash_element_matrix(&vec![elems.to_vec()]).unwrap();
        Self::Digest::from(&digest)
    }

    /// Returns a hash of two digests. This method is intended for use in construction of
    /// Merkle trees.
    fn merge(values: &[Self::Digest; 2]) -> Self::Digest {
        let hasher = Poseidon::new();
        let inp = vec![values[0].into(), values[1].into()];
        let init_state = Fr::zero();
        Self::Digest::from(&hasher.hash(&inp, &init_state).unwrap())
    }

    /// Returns hash(`seed` || `value`). This method is intended for use in PRNG and PoW contexts.
    fn merge_with_int(_seed: Self::Digest, _value: u64) -> Self::Digest {
        panic!("Unimplemented method");
    }
}
*/

#[cfg(test)]
mod tests {
    use crate::linearhash_bn128::LinearHashBN128;
    use crate::poseidon_bn128::{Fr, Poseidon};
    use ff::*;
    use winter_math::fields::f64::BaseElement;
    use winter_math::StarkField;

    #[test]
    fn test_linearhash_bn128() {
        let inputs: Vec<_> = (0..100).collect::<Vec<u64>>();
        let inputs: Vec<Vec<BaseElement>> = inputs
            .iter()
            .map(|e: &u64| {
                vec![
                    BaseElement::from(e.clone()),
                    BaseElement::from(e * 1000),
                    BaseElement::from(e * 1000000),
                ]
            })
            .collect();

        let st = LinearHashBN128::new().hash_element_matrix(&inputs).unwrap();
        assert_eq!(
            st.to_string(),
            "Fr(0x29c2ac38b7b8d18b9c1b575369cb4ab930ef71ebd5e4631b3916360233a29cae)",
        );
    }
}
