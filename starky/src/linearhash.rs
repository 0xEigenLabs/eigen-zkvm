#![allow(non_snake_case)]
use crate::errors::Result;
use crate::poseidon_opt::Poseidon;
use crate::ElementDigest;
//use rayon::prelude::*;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;

#[derive(Default)]
pub struct LinearHash {
    h: Poseidon,
}

impl LinearHash {
    pub fn new() -> Self {
        LinearHash { h: Poseidon::new() }
    }

    pub fn hash(
        &self,
        columns: &Vec<Vec<BaseElement>>,
        batch_size: usize,
    ) -> Result<ElementDigest> {
        let mut flatvals: Vec<BaseElement> = vec![];

        for col in columns.iter() {
            for elem in col.iter() {
                flatvals.push(*elem);
            }
        }

        let mut bs = batch_size;
        if bs == 0 {
            bs = core::cmp::max(8, (flatvals.len() + 3) / 4);
        }

        let mut st = [BaseElement::ZERO; 4];
        if flatvals.len() <= 4 {
            for (i, v) in flatvals.iter().enumerate() {
                st[i] = *v;
            }
            return Ok(ElementDigest::from(st));
        }

        let hsz = (flatvals.len() + bs - 1) / bs;
        let mut hashes: Vec<BaseElement> = vec![BaseElement::ZERO; hsz * 4];
        // NOTE flatsvals.len <= hashes.len
        hashes
            .chunks_mut(hsz)
            .zip(flatvals.chunks(bs))
            .for_each(|(outs, inps)| {
                let hv: [BaseElement; 4] = self._hash(inps).unwrap().into();
                outs[0..hv.len()].copy_from_slice(&hv);
            });

        if hashes.len() <= 4 {
            for (i, v) in hashes.iter().enumerate() {
                st[i] = *v;
            }
            Ok(ElementDigest::from(st))
        } else {
            self._hash(&hashes)
        }
    }

    pub fn _hash(&self, flatvals: &[BaseElement]) -> Result<ElementDigest> {
        let mut st = [BaseElement::ZERO; 4];
        if flatvals.len() <= 4 {
            for (i, v) in flatvals.iter().enumerate() {
                st[i] = *v;
            }
            return Ok(ElementDigest::from(st));
        }

        let mut inhashes: Vec<BaseElement> = vec![];
        for v in flatvals.iter() {
            inhashes.push(*v);
            if inhashes.len() == 8 {
                let t = self.h.hash(&inhashes, &st, 4).unwrap();
                st.copy_from_slice(&t);
                inhashes = vec![];
            }
        }
        if inhashes.len() > 0 {
            while inhashes.len() < 8 {
                inhashes.push(BaseElement::ZERO);
            }
            let t = self.h.hash(&inhashes, &st, 4).unwrap();
            st.copy_from_slice(&t);
        }
        Ok(ElementDigest::from(st))
    }
}

#[cfg(test)]
mod tests {
    use crate::digest_bn128::ElementDigest;
    use crate::linearhash::LinearHash;
    use winter_math::fields::f64::BaseElement;

    #[test]
    fn test_linearhash_gl_hash() {
        let lh = LinearHash::new();
        let raw_inputs = (1u32..28)
            .collect::<Vec<u32>>()
            .chunks(3)
            .collect::<Vec<&[u32]>>()
            .iter()
            .map(|ea| {
                let mut res: Vec<BaseElement> = vec![];
                for e in ea.iter() {
                    res.push(BaseElement::from(*e));
                }
                res
            })
            .collect::<Vec<Vec<BaseElement>>>();

        let res = lh.hash(&raw_inputs, 0).unwrap();
        let expected = ElementDigest::from([
            BaseElement::from(17618903473682537397u64),
            BaseElement::from(11844743283521766961u64),
            BaseElement::from(185773432536380223u64),
            BaseElement::from(6083210164459944430u64),
        ]);
        assert_eq!(expected, res);
    }

    #[test]
    fn test_linearhash_corner_case() {
        let lh = LinearHash::new();
        let raw_inputs = (1u32..4)
            .collect::<Vec<u32>>()
            .chunks(3)
            .collect::<Vec<&[u32]>>()
            .iter()
            .map(|ea| {
                let mut res: Vec<BaseElement> = vec![];
                for e in ea.iter() {
                    res.push(BaseElement::from(*e));
                }
                res
            })
            .collect::<Vec<Vec<BaseElement>>>();

        let res = lh.hash(&raw_inputs, 0).unwrap();
        let expected = ElementDigest::from([
            BaseElement::from(1u32),
            BaseElement::from(2u32),
            BaseElement::from(3u32),
            BaseElement::from(0u32),
        ]);
        assert_eq!(expected, res);
    }
}
