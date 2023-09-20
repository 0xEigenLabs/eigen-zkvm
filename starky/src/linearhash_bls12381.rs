#![allow(non_snake_case)]
use crate::errors::Result;
use crate::field_bls12381::{Fr, FrRepr};
use crate::poseidon_bls12381_opt::Poseidon;
use crate::traits::MTNodeType;
use crate::ElementDigest;
use ff::*;
//use rayon::prelude::*;
// ToDo: how to make sure the OFFSET constant is correctly initialized
use crate::constant::{BLS_OFFSET_2_192,BLS_OFFSET_2_128, BLS_OFFSET_2_64};
use plonky::field_gl::Fr as FGL;

#[derive(Default)]
pub struct LinearHashBLS12381 {
    h: Poseidon,
}
const ElementSize:usize = 6;
impl LinearHashBLS12381 {
    pub fn new() -> Self {
        LinearHashBLS12381 { h: Poseidon::new() }
    }

    pub fn hash_element_matrix(&self, columns: &Vec<Vec<FGL>>) -> Result<Fr> {
        let mut st = Fr::zero();
        let mut vals3: Vec<Fr> = vec![];

        let mut acc = Fr::zero();
        let mut accN = 0;

        for col in columns.iter() {
            for elem in col.iter() {
                let mut e = Fr::from_repr(FrRepr::from(elem.as_int()))?;
                if accN == 1 {
                    e.mul_assign(&BLS_OFFSET_2_64);
                } else if accN == 2 {
                    e.mul_assign(&BLS_OFFSET_2_128);
                } else if accN == 3 { 
                    e.mul_assign(&BLS_OFFSET_2_192);
                }
                // TODO: how to deal mul
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
    #[inline(always)]
    pub fn to_bls12381_mont(st64: [FGL; 6]) -> [FGL; 6] {
        // write:
        // x
        let bn: Fr = Fr(ElementDigest::<6>::new(&st64).as_scalar::<Fr>());
        let bn_mont = match Fr::from_repr(bn.into_raw_repr()) {
            Ok(x) => x,
            _ => {
                //cornor case: x > MODULUS
                let mut r = Fr(bn.into_raw_repr());
                // 2^381 mod p.
                const R2: FrRepr = FrRepr([
                    1997599621687373223u64,
                    6052339484930628067u64,
                    10108755138030829701u64,
                    150537098327114917u64,
                ]);
                r.mul_assign(&Fr(R2));
                r
            }
        };
        ElementDigest::<6>::from_scalar(&bn_mont)
            .as_elements()
            .try_into()
            .unwrap()
    }

    #[inline(always)]
    pub fn hash_node(
        &self,
        elems: &[ElementDigest<ElementSize>],
        init_state: &Fr,
    ) -> Result<ElementDigest<ElementSize>> {
        assert_eq!(elems.len(), 16);
        let elems = elems
            .iter()
            .map(|e| Fr((*e).as_scalar::<Fr>()))
            .collect::<Vec<Fr>>();
        let digest = self.h.hash(&elems, init_state)?;
        Ok(ElementDigest::<ElementSize>::from_scalar(&digest))
    }

    pub fn hash_element_array(&self, vals: &[FGL]) -> Result<ElementDigest<ElementSize>> {
        let mut st64 = [FGL::ZERO; 6];
        let mut digest: Fr = Fr::zero();
        if vals.len() <= 6 {
            for (i, v) in vals.iter().enumerate() {
                st64[i] = *v;
            }
            let gl_mont = Self::to_bls12381_mont(st64);
            return Ok(ElementDigest::<6>::new(&gl_mont));
        }

        // group into 3 * ElementSize
        let mut tmp_buf = vec![Fr::zero(); (vals.len() - 1) / 3 + 1];
        vals.chunks(3)
            .zip(tmp_buf.iter_mut())
            .for_each(|(ein, eout)| {
                // padding zero to ElementSize
                let mut ein_ElementSize = [FGL::ZERO; ElementSize];
                ein_ElementSize[..ein.len()].copy_from_slice(ein);
                *eout = crate::digest::to_bls12381(&ein_ElementSize);
            });

        // hash on each 16
        for i in (0..tmp_buf.len()).step_by(16) {
            let in_sz = std::cmp::min(16, tmp_buf.len() - i);
            digest = self.h.hash(&tmp_buf[i..(i + in_sz)].to_vec(), &digest)?;
        }

        Ok(ElementDigest::<ElementSize>::from_scalar(&digest))
    }
}