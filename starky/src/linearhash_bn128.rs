#![allow(non_snake_case)]
use crate::field_bn128::Fr;
use crate::poseidon_bn128::Poseidon;
use crate::ElementDigest;
use ff::*;
use winter_math::fields::f64::BaseElement;
use winter_math::{FieldElement, StarkField};

use crate::errors::Result;

#[derive(Default)]
pub struct LinearHashBN128 {
    h: Poseidon,
}

use crate::constant::*;

impl LinearHashBN128 {
    pub fn new() -> Self {
        LinearHashBN128 { h: Poseidon::new() }
    }

    /// used for hash leaves only, converting element from GL to BN128
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
    pub fn to_bn128_mont(st64: [BaseElement; 4]) -> [BaseElement; 4] {
        let bn: Fr = ElementDigest::new(st64).into();
        let bn_mont = Fr::from_repr(bn.into_raw_repr()).unwrap();
        ElementDigest::from(&bn_mont).into()
    }

    pub fn hash_node(&self, elems: &[ElementDigest], init_state: &Fr) -> Result<ElementDigest> {
        assert_eq!(elems.len(), 16);
        let elems = elems.iter().map(|e| e.clone().into()).collect::<Vec<Fr>>();
        let digest = self.h.hash(&elems, init_state)?;
        Ok(ElementDigest::from(&digest))
    }

    fn hash_element_block(&self, elems: &[BaseElement], init_state: &Fr) -> Result<Fr> {
        //println!("hash_element_block size: {}", elems.len());
        let elems = elems
            .chunks(4)
            .map(|e| ElementDigest::to_BN128(e.try_into().unwrap()))
            .collect::<Vec<Fr>>();
        //println!("\nelem.length {:?}, {:?}", elems.len(), elems);
        Ok(self.h.hash(&elems, init_state)?)
    }

    pub fn hash_element_array(&self, vals: &Vec<BaseElement>) -> Result<ElementDigest> {
        let mut st64 = [BaseElement::ZERO; 4];
        let mut in64: [BaseElement; 64] = [BaseElement::ZERO; 64];
        let mut digest: Fr = Fr::zero();
        //println!("hash_element_array size: {}", vals.len());
        if vals.len() <= 4 {
            for (i, v) in vals.iter().enumerate() {
                st64[i] = *v;
            }
            let gl_mont = Self::to_bn128_mont(st64);
            return Ok(ElementDigest::from(gl_mont));
        }

        let mut p = 0;

        for (i, val) in vals.iter().enumerate() {
            in64[p] = *val;
            p += 1;
            if p == 16 * 4 {
                digest = self.hash_element_block(&in64[..], &digest)?;
                p = 0;
            }
            if i % 3 == 2 {
                in64[p] = BaseElement::ZERO;
                p += 1;
                if p == 16 * 4 {
                    digest = self.hash_element_block(&in64[..], &digest)?;
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
            digest = self.hash_element_block(&in64[..(nLast * 4)], &digest)?;
        }
        Ok(ElementDigest::from(&digest))
    }
}

#[cfg(test)]
mod tests {
    use crate::field_bn128::Fr;
    use crate::linearhash_bn128::LinearHashBN128;
    use crate::poseidon_bn128::Poseidon;
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

    #[test]
    fn test_linearhash_corner_case() {
        let input = vec![
            BaseElement::from(6188675464075253840u64),
            BaseElement::from(2608530331018891925u64),
        ];
        crate::helper::pretty_print_array(&input);

        let mut lh = LinearHashBN128::new();
        let result = lh.hash_element_array(&input).unwrap();
        println!("out {}", result);
        assert_eq!(result.0[0], BaseElement::from(15714769047018385385u64));
        assert_eq!(result.0[1], BaseElement::from(14080511166848616671u64));
        assert_eq!(result.0[2], BaseElement::from(11411897157942048316u64));
        assert_eq!(result.0[3], BaseElement::from(1802287360671936077u64));

        let input = vec![
            BaseElement::from(18440682777423237490u64),
            BaseElement::from(1156220815552880681u64),
        ];

        let result = lh.hash_element_array(&input).unwrap();
        println!("out {}", result);
        assert_eq!(result.0[0], BaseElement::from(12850950522295690944u64));
        assert_eq!(result.0[1], BaseElement::from(15045028186447136619u64));
        assert_eq!(result.0[2], BaseElement::from(11701297961637547631u64));
        assert_eq!(result.0[3], BaseElement::from(875058675367281598u64));
    }
}
