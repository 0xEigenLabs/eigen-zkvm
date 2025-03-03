#![allow(non_snake_case)]
#[cfg(all(
    target_feature = "avx2",
    not(all(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    ))
))]
use crate::arch::x86_64::avx2_poseidon_gl::Poseidon;
#[cfg(all(
    target_feature = "avx512bw",
    target_feature = "avx512cd",
    target_feature = "avx512dq",
    target_feature = "avx512f",
    target_feature = "avx512vl"
))]
use crate::arch::x86_64::avx512_poseidon_gl::Poseidon;
#[cfg(not(any(
    target_feature = "avx2",
    target_feature = "avx512bw",
    target_feature = "avx512cd",
    target_feature = "avx512dq",
    target_feature = "avx512f",
    target_feature = "avx512vl"
)))]
use crate::poseidon_opt::Poseidon;
use crate::traits::MTNodeType;
use crate::ElementDigest;
use anyhow::Result;
use fields::field_gl::Fr as FGL;
use profiler_macro::time_profiler;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearHash {
    h: Poseidon,
}

impl LinearHash {
    pub fn new() -> Self {
        LinearHash { h: Poseidon::new() }
    }

    #[time_profiler()]
    #[cfg(not(any(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    )))]
    pub fn hash_element_matrix(
        &self,
        vals: &[Vec<FGL>],
        batch_size: usize,
    ) -> Result<ElementDigest<4, FGL>> {
        let mut flatvals = vec![FGL::default(); vals.len() * vals[0].len()];

        flatvals.par_chunks_mut(vals[0].len()).zip(vals.par_iter()).for_each(
            |(flat_chunk, col)| {
                flat_chunk.copy_from_slice(col);
            },
        );

        self.hash(&flatvals, batch_size)
    }

    #[cfg(not(any(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    )))]
    pub fn hash(&self, flatvals: &[FGL], batch_size: usize) -> Result<ElementDigest<4, FGL>> {
        let mut bs = batch_size;
        if bs == 0 {
            bs = core::cmp::max(8, flatvals.len().div_ceil(4));
        }

        let mut st = [FGL::ZERO; 4];
        if flatvals.len() <= 4 {
            for (i, v) in flatvals.iter().enumerate() {
                st[i] = *v;
            }
            return Ok(ElementDigest::<4, FGL>::new(&st));
        }

        let hsz = flatvals.len().div_ceil(bs);
        let mut hashes: Vec<FGL> = vec![FGL::ZERO; hsz * 4];
        // NOTE flatsvals.len <= hashes.len
        hashes.chunks_mut(4).zip(flatvals.chunks(bs)).for_each(|(outs, inps)| {
            let hv = self._hash(inps).unwrap();
            let hv: &[FGL] = hv.as_elements();
            outs[0..hv.len()].copy_from_slice(hv);
        });

        if hashes.len() <= 4 {
            for (i, v) in hashes.iter().enumerate() {
                st[i] = *v;
            }
            Ok(ElementDigest::<4, FGL>::new(&st))
        } else {
            self._hash(&hashes)
        }
    }

    #[cfg(not(any(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    )))]
    pub fn _hash(&self, flatvals: &[FGL]) -> Result<ElementDigest<4, FGL>> {
        let mut st = [FGL::ZERO; 4];
        if flatvals.len() <= 4 {
            for (i, v) in flatvals.iter().enumerate() {
                st[i] = *v;
            }
            return Ok(ElementDigest::<4, FGL>::new(&st));
        }

        let mut inhashes: Vec<FGL> = vec![];
        for v in flatvals.iter() {
            inhashes.push(*v);
            if inhashes.len() == 8 {
                let t = self.h.hash(&inhashes, &st, 4).unwrap();
                st.copy_from_slice(&t);
                inhashes = vec![];
            }
        }
        if !inhashes.is_empty() {
            while inhashes.len() < 8 {
                inhashes.push(FGL::ZERO);
            }
            let t = self.h.hash(&inhashes, &st, 4).unwrap();
            st.copy_from_slice(&t);
        }
        Ok(ElementDigest::<4, FGL>::new(&st))
    }

    #[cfg(all(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    ))]
    pub fn hash_element_matrix(
        &self,
        vals: &[Vec<FGL>],
        batch_size: usize,
    ) -> Result<ElementDigest<4, FGL>> {
        let mut flatvals = vec![FGL::default(); vals.len() * vals[0].len()];

        flatvals.par_chunks_mut(vals[0].len()).zip(vals.par_iter()).for_each(
            |(flat_chunk, col)| {
                flat_chunk.copy_from_slice(col);
            },
        );

        let flatvals_1: Vec<FGL> = [flatvals.clone(), flatvals.clone()].concat();

        let hash_result = self.hash(&flatvals_1, batch_size).unwrap()[0];
        Ok(hash_result)
    }

    #[cfg(all(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    ))]
    pub fn hash(&self, flatvals: &[FGL], batch_size: usize) -> Result<[ElementDigest<4, FGL>; 2]> {
        let mid = flatvals.len() / 2;
        let flatvals0 = &flatvals[..mid];
        let flatvals1 = &flatvals[mid..];

        let mut bs = batch_size;
        if bs == 0 {
            bs = core::cmp::max(8, (mid + 3) / 4);
        }

        let mut st0 = [FGL::ZERO; 4];
        let mut st1 = [FGL::ZERO; 4];
        if mid <= 4 {
            for (i, v) in flatvals0.iter().enumerate() {
                st0[i] = *v;
            }
            for (i, v) in flatvals1.iter().enumerate() {
                st1[i] = *v;
            }
            return Ok([ElementDigest::<4, FGL>::new(&st0), ElementDigest::<4, FGL>::new(&st1)]);
        }

        let hsz = (mid + bs - 1) / bs;
        let mut hashes: Vec<FGL> = vec![FGL::ZERO; hsz * 4 * 2];
        // NOTE flatsvals.len <= hashes.len
        hashes.chunks_mut(8).zip(flatvals0.chunks(bs)).zip(flatvals1.chunks(bs)).for_each(
            |((outs, chunk0), chunk1)| {
                let mut inps = Vec::new();
                inps.extend_from_slice(chunk0);
                inps.extend_from_slice(chunk1);
                let hash_result = self._hash(inps.as_slice()).unwrap();
                outs.copy_from_slice(&hash_result);
            },
        );

        if hashes.len() <= 8 {
            let mid = hashes.len() / 2;
            for (i, &v) in hashes.iter().take(mid).enumerate() {
                st0[i % 4] = v;
            }
            for (i, &v) in hashes.iter().skip(mid).enumerate() {
                st1[i % 4] = v;
            }
            return Ok([ElementDigest::<4, FGL>::new(&st0), ElementDigest::<4, FGL>::new(&st1)]);
        } else {
            let mut hash: Vec<FGL> = Vec::with_capacity(hashes.len());
            for chunk in hashes.chunks(8) {
                let (first_half, _) = chunk.split_at(4);
                hash.extend_from_slice(first_half);
            }
            for chunk in hashes.chunks(8) {
                let (_, second_half) = chunk.split_at(4);
                hash.extend_from_slice(second_half);
            }
            let tmp = self._hash(&hash).unwrap();
            return Ok([
                ElementDigest::<4, FGL>::new(&tmp[0..4]),
                ElementDigest::<4, FGL>::new(&tmp[4..8]),
            ]);
        }
    }

    #[cfg(all(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    ))]
    pub fn _hash(&self, flatvals: &[FGL]) -> Result<[FGL; 8]> {
        let mid = flatvals.len() / 2;
        let flatvals0 = &flatvals[..mid];
        let flatvals1 = &flatvals[mid..];
        let mut st0 = [FGL::ZERO; 4];
        let mut st1 = [FGL::ZERO; 4];
        if mid <= 4 {
            for (i, v) in flatvals0.iter().enumerate() {
                st0[i] = *v;
            }
            for (i, v) in flatvals1.iter().enumerate() {
                st1[i] = *v;
            }
            let result = [st0[0], st0[1], st0[2], st0[3], st1[0], st1[1], st1[2], st1[3]];
            return Ok(result);
        }
        let mut count = 0;
        let mut st = [FGL::ZERO; 8];
        let mut inhashes: Vec<FGL> = vec![];

        for v in flatvals0.iter() {
            inhashes.push(*v);
            if inhashes.len() == 8 {
                let start = count * 8;
                let mid = start + 4;
                let end = start + 8;
                let first_half = &flatvals1[start..mid];
                inhashes.splice(4..4, first_half.iter().cloned());
                let second_half = &flatvals1[mid..end];
                inhashes.extend_from_slice(second_half);
                let t = self.h.hash(&inhashes, &st, 8).unwrap();
                st.copy_from_slice(&t);
                inhashes.clear();
                count += 1;
            }
        }

        if !inhashes.is_empty() {
            while inhashes.len() < 8 {
                inhashes.push(FGL::ZERO);
            }
            inhashes.extend_from_slice(&flatvals1[count * 8..]);
            while inhashes.len() < 16 {
                inhashes.push(FGL::ZERO);
            }
            let middle_chunk = inhashes.splice(4..8, vec![]).collect::<Vec<_>>();
            inhashes.splice(8..8, middle_chunk.iter().cloned());
            let t = self.h.hash(&inhashes, &st, 8).unwrap();
            st.copy_from_slice(&t);
        }
        Ok(st)
    }
}

#[cfg(test)]
mod tests {
    use crate::digest::ElementDigest;
    use crate::linearhash::LinearHash;
    use crate::traits::MTNodeType;
    use fields::field_gl::Fr as FGL;

    #[test]
    fn test_linearhash_gl_hash() {
        let lh = LinearHash::new();
        let raw_inputs = (1u64..28)
            .collect::<Vec<u64>>()
            .chunks(3)
            .collect::<Vec<&[u64]>>()
            .iter()
            .map(|ea| {
                let mut res: Vec<FGL> = vec![];
                for e in ea.iter() {
                    res.push(FGL::from(*e));
                }
                res
            })
            .collect::<Vec<Vec<FGL>>>();

        let res = lh.hash_element_matrix(&raw_inputs, 0).unwrap();
        let expected = ElementDigest::<4, FGL>::new(&[
            FGL::from(17618903473682537397u64),
            FGL::from(11844743283521766961u64),
            FGL::from(185773432536380223u64),
            FGL::from(6083210164459944430u64),
        ]);
        assert_eq!(expected, res);
    }

    #[test]
    fn test_linearhash_corner_case() {
        let lh = LinearHash::new();
        let raw_inputs = (1u64..4)
            .collect::<Vec<u64>>()
            .chunks(3)
            .collect::<Vec<&[u64]>>()
            .iter()
            .map(|ea| {
                let mut res: Vec<FGL> = vec![];
                for e in ea.iter() {
                    res.push(FGL::from(*e));
                }
                res
            })
            .collect::<Vec<Vec<FGL>>>();

        let res = lh.hash_element_matrix(&raw_inputs, 0).unwrap();
        let expected = ElementDigest::<4, FGL>::new(&[
            FGL::from(1u64),
            FGL::from(2u64),
            FGL::from(3u64),
            FGL::from(0u64),
        ]);
        assert_eq!(expected, res);
    }
}
