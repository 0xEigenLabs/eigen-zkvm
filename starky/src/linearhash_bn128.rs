#![allow(non_snake_case)]
use crate::field_bn128::{Fr, FrRepr};
use crate::poseidon_bn128_opt::Poseidon;
use crate::traits::MTNodeType;
use crate::ElementDigest;
use anyhow::Result;
use ff::*;
use serde::{Deserialize, Serialize};
//use rayon::prelude::*;
use crate::constant::{OFFSET_2_128, OFFSET_2_64};
use fields::field_gl::Fr as FGL;

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearHashBN128 {
    h: Poseidon,
}

impl LinearHashBN128 {
    pub fn new() -> Self {
        LinearHashBN128 { h: Poseidon::new() }
    }

    pub fn hash_element_matrix(&self, columns: &[Vec<FGL>]) -> Result<Fr> {
        let mut st = Fr::zero();
        let mut vals3: Vec<Fr> = vec![];

        let mut acc = Fr::zero();
        let mut accN = 0;

        for col in columns.iter() {
            for elem in col.iter() {
                let mut e = Fr::from_repr(FrRepr::from(elem.as_int()))?;
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
        if vals3.is_empty() {
            return Ok(st);
        } else if vals3.len() == 1 {
            return Ok(vals3[0]);
        }
        let mut inHash: Vec<Fr> = vec![];

        for val3 in vals3.iter() {
            inHash.push(*val3);
            if inHash.len() == 16 {
                st = self.h.hash(&inHash, &st)?;
                inHash = vec![];
            }
        }
        if !inHash.is_empty() {
            st = self.h.hash(&inHash, &st)?;
        }
        Ok(st)
    }

    /// convert to BN128 in montgomery
    #[inline(always)]
    pub fn to_bn128_mont(st64: [FGL; 4]) -> [FGL; 4] {
        let bn: Fr = Fr(ElementDigest::<4, Fr>::new(&st64).as_scalar::<Fr>());
        let bn_mont = match Fr::from_repr(bn.into_raw_repr()) {
            Ok(x) => x,
            _ => {
                //cornor case: x > MODULUS
                let mut r = Fr(bn.into_raw_repr());
                // 2^128 mod p.
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
        ElementDigest::<4, Fr>::from_scalar(&bn_mont).as_elements().try_into().unwrap()
    }

    #[inline(always)]
    pub fn hash_node(
        &self,
        elems: &[ElementDigest<4, Fr>],
        init_state: &Fr,
    ) -> Result<ElementDigest<4, Fr>> {
        assert_eq!(elems.len(), 16);
        let elems = elems.iter().map(|e| Fr((*e).as_scalar::<Fr>())).collect::<Vec<Fr>>();
        let digest = self.h.hash(&elems, init_state)?;
        Ok(ElementDigest::<4, Fr>::from_scalar(&digest))
    }

    pub fn hash_element_array(&self, vals: &[FGL]) -> Result<ElementDigest<4, Fr>> {
        let mut st64 = [FGL::ZERO; 4];
        let mut digest: Fr = Fr::zero();
        if vals.len() <= 4 {
            for (i, v) in vals.iter().enumerate() {
                st64[i] = *v;
            }
            let gl_mont = Self::to_bn128_mont(st64);
            return Ok(ElementDigest::<4, Fr>::new(&gl_mont));
        }

        // group into 3 * 4
        let mut tmp_buf = vec![Fr::zero(); (vals.len() - 1) / 3 + 1];
        vals.chunks(3).zip(tmp_buf.iter_mut()).for_each(|(ein, eout)| {
            // padding zero to 4
            let mut ein_4 = [FGL::ZERO; 4];
            ein_4[..ein.len()].copy_from_slice(ein);
            *eout = crate::digest::to_bn128(&ein_4);
        });

        // hash on each 16
        for i in (0..tmp_buf.len()).step_by(16) {
            let in_sz = std::cmp::min(16, tmp_buf.len() - i);
            digest = self.h.hash(&tmp_buf[i..(i + in_sz)], &digest)?;
        }

        Ok(ElementDigest::<4, Fr>::from_scalar(&digest))
    }
}

#[cfg(test)]
mod tests {
    use crate::linearhash_bn128::LinearHashBN128;
    use fields::field_gl::Fr as FGL;

    #[test]
    fn test_linearhash_matrix_bn128() {
        let inputs: Vec<_> = (0..100).collect::<Vec<u64>>();
        let inputs: Vec<Vec<FGL>> = inputs
            .iter()
            .map(|e: &u64| vec![FGL::from(*e), FGL::from(e * 1000), FGL::from(e * 1000000)])
            .collect();

        let st = LinearHashBN128::new().hash_element_matrix(&inputs).unwrap();
        assert_eq!(
            st.to_string(),
            "Fr(0x29c2ac38b7b8d18b9c1b575369cb4ab930ef71ebd5e4631b3916360233a29cae)",
        );
    }

    #[test]
    fn test_linearhash_corner_case() {
        let input = vec![FGL::from(6188675464075253840u64), FGL::from(2608530331018891925u64)];

        let lh = LinearHashBN128::new();
        let result = lh.hash_element_array(&input).unwrap();
        log::trace!("out {}", result);
        assert_eq!(result.0[0], FGL::from(15714769047018385385u64));
        assert_eq!(result.0[1], FGL::from(14080511166848616671u64));
        assert_eq!(result.0[2], FGL::from(11411897157942048316u64));
        assert_eq!(result.0[3], FGL::from(1802287360671936077u64));

        let input = vec![FGL::from(18440682777423237490u64), FGL::from(1156220815552880681u64)];

        let result = lh.hash_element_array(&input).unwrap();
        log::trace!("out {}", result);
        assert_eq!(result.0[0], FGL::from(12850950522295690944u64));
        assert_eq!(result.0[1], FGL::from(15045028186447136619u64));
        assert_eq!(result.0[2], FGL::from(11701297961637547631u64));
        assert_eq!(result.0[3], FGL::from(875058675367281598u64));
    }
}
