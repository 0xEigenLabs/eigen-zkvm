#![allow(dead_code)]

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
use crate::constant::{get_max_workers, MAX_OPS_PER_THREAD, MIN_OPS_PER_THREAD};
use crate::digest::ElementDigest;
use crate::f3g::F3G;
use crate::linearhash::LinearHash;
#[cfg(any(
    not(target_feature = "avx2"),
    all(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    )
))]
use crate::poseidon_opt::Poseidon;
use crate::traits::MTNodeType;
use crate::traits::MerkleTree;
use anyhow::{bail, Result};
use fields::field_gl::Fr as FGL;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MerkleTreeGL {
    pub elements: Vec<FGL>,
    pub width: usize,
    pub height: usize,
    pub nodes: Vec<ElementDigest<4, FGL>>,
    h: LinearHash,
    poseidon: Poseidon,
}

fn get_n_nodes(n_: usize) -> usize {
    let mut n = n_;
    let mut next_n = (n - 1) / 2 + 1;
    let mut acc = next_n * 2;
    while n > 1 {
        n = next_n;
        next_n = (n - 1) / 2 + 1;
        if n > 1 {
            acc += next_n * 2;
        } else {
            acc += 1;
        }
    }
    acc
}

impl MerkleTreeGL {
    fn merkle_gen_merkle_proof(&self, idx: usize, offset: usize, n: usize) -> Vec<Vec<FGL>> {
        if n <= 1 {
            return vec![];
        }
        let next_idx = idx >> 1;
        let si = idx ^ 1;
        let sib = self.nodes[offset + si].as_elements().to_vec();

        let next_n = (n - 1) / 2 + 1;
        let mut result = vec![sib];
        result.append(&mut self.merkle_gen_merkle_proof(next_idx, offset + next_n * 2, next_n));
        result
    }

    #[inline]
    fn merklize_level(&mut self, p_in: usize, n_ops: usize, p_out: usize) -> Result<()> {
        let mut n_ops_per_thread = (n_ops - 1) / (get_max_workers() * 2) + 1;
        if n_ops_per_thread < MIN_OPS_PER_THREAD {
            n_ops_per_thread = MIN_OPS_PER_THREAD;
        }

        let buff = &self.nodes[p_in..(p_in + n_ops * 2)];
        let nodes = buff
            .par_chunks(2 * n_ops_per_thread)
            .enumerate()
            .map(|(i, bb)| self.do_merklize_level(bb, i, n_ops).unwrap())
            .reduce(
                Vec::<ElementDigest<4, FGL>>::new,
                |mut a: Vec<ElementDigest<4, FGL>>, mut b: Vec<ElementDigest<4, FGL>>| {
                    a.append(&mut b);
                    a
                },
            );

        let out = &mut self.nodes[p_out..(p_out + n_ops)];
        out.iter_mut().zip(nodes).for_each(|(nout, nin)| *nout = nin);
        Ok(())
    }

    #[cfg(not(any(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    )))]
    fn do_merklize_level(
        &self,
        buff_in: &[ElementDigest<4, FGL>],
        _st_i: usize,
        _st_n: usize,
    ) -> Result<Vec<ElementDigest<4, FGL>>> {
        log::trace!(
            "merklizing GL hash start.... {}/{}, buff size {}",
            _st_i,
            _st_n,
            buff_in.len()
        );
        let n_ops = buff_in.len() / 2;
        let mut buff_out64: Vec<ElementDigest<4, FGL>> =
            vec![ElementDigest::<4, FGL>::default(); n_ops];
        buff_out64.iter_mut().zip(0..n_ops).for_each(|(out, i)| {
            let mut two = [FGL::ZERO; 8];
            let one: &[FGL] = buff_in[i * 2].as_elements();
            two[0..4].copy_from_slice(one);
            let one: &[FGL] = buff_in[i * 2 + 1].as_elements();
            two[4..8].copy_from_slice(one);
            *out = self.h.hash(&two, 0).unwrap();
        });
        Ok(buff_out64)
    }

    #[cfg(all(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    ))]
    fn do_merklize_level(
        &self,
        buff_in: &[ElementDigest<4, FGL>],
        _st_i: usize,
        _st_n: usize,
    ) -> Result<Vec<ElementDigest<4, FGL>>> {
        log::trace!(
            "merklizing GL hash start.... {}/{}, buff size {}",
            _st_i,
            _st_n,
            buff_in.len()
        );
        let n_ops = buff_in.len() / 4;
        let mut buff_out64: Vec<ElementDigest<4, FGL>> =
            vec![ElementDigest::<4, FGL>::default(); buff_in.len() / 2];
        let process = |chunk: &[ElementDigest<4, FGL>], four: &mut [FGL; 16]| {
            for (j, item) in chunk.iter().enumerate() {
                let one: &[FGL] = item.as_elements();
                four[j * 4..(j + 1) * 4].copy_from_slice(one);
            }
            self.h.hash(four, 0).unwrap()
        };

        let mut four = [FGL::ZERO; 16];
        if n_ops == 0 {
            let hash_result = process(&buff_in[..2], &mut four);
            buff_out64[0] = hash_result[0];
        } else {
            for i in 0..n_ops {
                let hash_result = process(&buff_in[i * 4..i * 4 + 4], &mut four);
                buff_out64[i * 2] = hash_result[0];
                buff_out64[i * 2 + 1] = hash_result[1];
            }
            if buff_in.len() % 4 != 0 {
                let hash_result = process(&buff_in[buff_in.len() - 2..], &mut four);
                buff_out64[n_ops * 2] = hash_result[0];
            }
        }
        Ok(buff_out64)
    }

    fn merkle_calculate_root_from_proof(
        &self,
        mp: &[Vec<FGL>],
        idx: usize,
        value: &ElementDigest<4, FGL>,
        offset: usize,
    ) -> Result<ElementDigest<4, FGL>> {
        if mp.len() == offset {
            return Ok(*value);
        }
        let cur_idx = idx & 1;
        let next_idx = idx / 2;
        let init = [FGL::ZERO; 4];

        let mut inhash = vec![FGL::ZERO; 8];
        if cur_idx == 0 {
            let one = value.as_elements();
            inhash[0..4].copy_from_slice(one);
            inhash[4..(4 + 4)].copy_from_slice(&mp[offset][..4]);
        } else {
            inhash[..4].copy_from_slice(&mp[offset][..4]);
            let one = value.as_elements();
            inhash[4..8].copy_from_slice(one);
        }
        let next = self.poseidon.hash(&inhash, &init, 4)?;
        let next_value = ElementDigest::<4, FGL>::new(&next);
        self.merkle_calculate_root_from_proof(mp, next_idx, &next_value, offset + 1)
    }

    #[cfg(not(any(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    )))]
    fn calculate_root_from_group_proof(
        &self,
        mp: &[Vec<FGL>],
        idx: usize,
        vals: &[FGL],
    ) -> Result<ElementDigest<4, FGL>> {
        let h = self.h.hash(vals, 0)?;
        self.merkle_calculate_root_from_proof(mp, idx, &h, 0)
    }

    #[cfg(all(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    ))]
    fn calculate_root_from_group_proof(
        &self,
        mp: &[Vec<FGL>],
        idx: usize,
        vals: &[FGL],
    ) -> Result<ElementDigest<4, FGL>> {
        let mut vals_0: Vec<FGL> = Vec::with_capacity(vals.len() * 2);
        vals_0.extend_from_slice(vals);
        vals_0.extend_from_slice(vals);
        let h = self.h.hash(&vals_0, 0)?;
        self.merkle_calculate_root_from_proof(mp, idx, &h[0], 0)
    }
}

impl MerkleTree for MerkleTreeGL {
    type BaseField = FGL;
    type MTNode = ElementDigest<4, FGL>;
    type ExtendField = F3G;
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            elements: Vec::new(),
            h: LinearHash::new(),
            width: 0,
            height: 0,
            poseidon: Poseidon::new(),
        }
    }

    fn element_size(&self) -> usize {
        self.elements.len()
    }

    fn to_extend(&self, p_be: &mut Vec<F3G>) {
        assert_eq!(p_be.len(), self.elements.len());
        p_be.par_iter_mut().zip(&self.elements).for_each(|(be_out, f3g_in)| {
            *be_out = F3G::from(*f3g_in);
        });
    }

    // For any MTNode in GL MerkleTree, it's a format of [val, 0, 0, 0]
    fn to_basefield(node: &Self::MTNode) -> Vec<Self::BaseField> {
        vec![node.as_elements().to_vec()[0]]
    }

    fn from_basefield(node: &FGL) -> Self::MTNode {
        Self::MTNode::new(&[*node, FGL::ZERO, FGL::ZERO, FGL::ZERO])
    }

    #[cfg(not(any(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    )))]
    fn merkelize(&mut self, buff: Vec<FGL>, width: usize, height: usize) -> Result<()> {
        let max_workers = get_max_workers();

        let mut n_per_thread_f = (height - 1) / max_workers + 1;

        let div = core::cmp::max(width / 8, 1);
        let max_corrected = MAX_OPS_PER_THREAD / div;
        let min_corrected = MIN_OPS_PER_THREAD / div;

        if n_per_thread_f > max_corrected {
            n_per_thread_f = max_corrected;
        }
        if n_per_thread_f < min_corrected {
            n_per_thread_f = min_corrected;
        }

        let mut nodes = vec![Self::MTNode::default(); get_n_nodes(height)];
        let now = Instant::now();
        if !buff.is_empty() {
            nodes
                .par_chunks_mut(n_per_thread_f)
                .zip(buff.par_chunks(n_per_thread_f * width))
                .for_each(|(out, bb)| {
                    let cur_n = bb.len() / width;
                    out.iter_mut().zip(0..cur_n).for_each(|(row_out, j)| {
                        let batch = &bb[(j * width)..((j + 1) * width)];
                        *row_out = self.h.hash(batch, 0).unwrap();
                    });
                });
        }
        log::trace!("linearhash time cost: {}", now.elapsed().as_secs_f64());

        // merklize level
        self.nodes = nodes;
        self.elements = buff;
        self.width = width;
        self.height = height;

        let mut n64: usize = height;
        let mut next_n64: usize = (n64 - 1) / 2 + 1;
        let mut p_in: usize = 0;
        let mut p_out: usize = p_in + next_n64 * 2;
        while n64 > 1 {
            let now = Instant::now();
            self.merklize_level(p_in, next_n64, p_out)?;
            log::trace!("merklize_level {} time cost: {}", next_n64, now.elapsed().as_secs_f64());
            n64 = next_n64;
            next_n64 = (n64 - 1) / 2 + 1;
            p_in = p_out;
            p_out = p_in + next_n64 * 2;
        }

        Ok(())
    }

    #[cfg(all(
        target_feature = "avx512bw",
        target_feature = "avx512cd",
        target_feature = "avx512dq",
        target_feature = "avx512f",
        target_feature = "avx512vl"
    ))]
    fn merkelize(&mut self, buff: Vec<FGL>, width: usize, height: usize) -> Result<()> {
        let max_workers = get_max_workers();

        let mut n_per_thread_f = (height - 1) / max_workers + 1;

        let div = core::cmp::max(width / 8, 1);
        let max_corrected = MAX_OPS_PER_THREAD / div;
        let min_corrected = MIN_OPS_PER_THREAD / div;

        if n_per_thread_f > max_corrected {
            n_per_thread_f = max_corrected;
        }
        if n_per_thread_f < min_corrected {
            n_per_thread_f = min_corrected;
        }

        let mut nodes = vec![Self::MTNode::default(); get_n_nodes(height)];
        let now = Instant::now();

        if !buff.is_empty() {
            nodes
                .par_chunks_mut(n_per_thread_f)
                .zip(buff.par_chunks(n_per_thread_f * width))
                .for_each(|(out, bb)| {
                    let cur_n = bb.len() / width / 2;
                    (0..cur_n).for_each(|j| {
                        let batch = &bb[(j * width * 2)..((j + 1) * width * 2)];
                        let hash_result = self.h.hash(batch, 0).unwrap();
                        let index = j * 2;
                        if index < out.len() && index + 1 < out.len() {
                            out[index] = hash_result[0];
                            out[index + 1] = hash_result[1];
                        }
                    });
                    if bb.len() % (width * 2) != 0 {
                        let remaining = &bb[cur_n * width * 2..];
                        let mut batch = vec![FGL::ZERO; width * 2];
                        batch[..remaining.len()].copy_from_slice(remaining);
                        batch[remaining.len()..].copy_from_slice(remaining);
                        let hash_result = self.h.hash(&batch, 0).unwrap();
                        out[cur_n * 2] = hash_result[0];
                    }
                });
        }

        log::trace!("linearhash time cost: {}", now.elapsed().as_secs_f64());

        // merklize level
        self.nodes = nodes;
        self.elements = buff;
        self.width = width;
        self.height = height;

        let mut n64: usize = height;
        let mut next_n64: usize = (n64 - 1) / 2 + 1;
        let mut p_in: usize = 0;
        let mut p_out: usize = p_in + next_n64 * 2;
        while n64 > 1 {
            let now = Instant::now();
            self.merklize_level(p_in, next_n64, p_out)?;
            log::trace!("merklize_level {} time cost: {}", next_n64, now.elapsed().as_secs_f64());
            n64 = next_n64;
            next_n64 = (n64 - 1) / 2 + 1;
            p_in = p_out;
            p_out = p_in + next_n64 * 2;
        }

        Ok(())
    }

    fn get_element(&self, idx: usize, sub_idx: usize) -> FGL {
        self.elements[self.width * idx + sub_idx]
    }

    // the path always returns 2-dim array likes [[x,x,x,x], ...]
    fn get_group_proof(&self, idx: usize) -> Result<(Vec<FGL>, Vec<Vec<FGL>>)> {
        if idx >= self.height {
            bail!("MerkleTreeError: access invalid node");
        }

        let v = (0..self.width).map(|i| self.get_element(idx, i)).collect::<Vec<_>>();
        let mp = self.merkle_gen_merkle_proof(idx, 0, self.height);
        Ok((v, mp))
    }

    fn eq_root(&self, r1: &Self::MTNode, r2: &Self::MTNode) -> bool {
        r1 == r2
    }

    fn verify_group_proof(
        &self,
        root: &Self::MTNode,
        mp: &[Vec<FGL>],
        idx: usize,
        group_elements: &[FGL],
    ) -> Result<bool> {
        let c_root = self.calculate_root_from_group_proof(mp, idx, group_elements)?;
        Ok(self.eq_root(root, &c_root))
    }

    fn root(&self) -> Self::MTNode {
        self.nodes[self.nodes.len() - 1]
    }
}

#[cfg(test)]
mod tests {
    use crate::merklehash::MerkleTreeGL;
    use crate::traits::MTNodeType;
    use crate::traits::MerkleTree;
    use fields::field_gl::Fr as FGL;
    use std::time::Instant;

    #[test]
    fn test_merklehash_gl_simple() {
        let n = 256;
        let idx = 3;
        let n_pols = 9;

        let mut cols: Vec<FGL> = vec![FGL::ZERO; n_pols * n];
        for i in 0..n {
            for j in 0..n_pols {
                cols[i * n_pols + j] = FGL::from((i + j * 1000) as u64);
            }
        }
        let start = Instant::now();
        let mut tree = MerkleTreeGL::new();
        tree.merkelize(cols, n_pols, n).unwrap();
        let (v, mp) = tree.get_group_proof(idx).unwrap();
        let root = tree.root();
        let duration = start.elapsed();
        println!("time: {:?}", duration);
        let re = root.as_elements();
        let expected = vec![
            FGL::from(11508832812350783315u64),
            FGL::from(5044133147279090978u64),
            FGL::from(6335412741057168694u64),
            FGL::from(12530816673814004438u64),
        ];
        assert_eq!(expected, re);

        assert!(tree.verify_group_proof(&root, &mp, idx, &v).unwrap());
    }

    #[test]
    fn test_merklehash_gl_small() {
        let n = 256;
        let idx = 3;
        let n_pols = 9;
        let mut pols: Vec<FGL> = vec![FGL::ZERO; n_pols * n];
        for i in 0..n {
            for j in 0..n_pols {
                pols[i * n_pols + j] = FGL::from((i + j * 1000) as u64);
            }
        }

        let mut tree = MerkleTreeGL::new();
        tree.merkelize(pols, n_pols, n).unwrap();
        let (group_elements, mp) = tree.get_group_proof(idx).unwrap();
        let root = tree.root();
        assert!(tree.verify_group_proof(&root, &mp, idx, &group_elements).unwrap());
    }

    #[test]
    fn test_merklehash_gl_not_power_of_2() {
        let n = 33;
        let idx = 32;
        let n_pols = 6;
        let mut pols: Vec<FGL> = vec![FGL::ZERO; n_pols * n];
        for i in 0..n {
            for j in 0..n_pols {
                pols[i * n_pols + j] = FGL::from((i + j * 1000) as u64);
            }
        }

        let mut tree = MerkleTreeGL::new();
        tree.merkelize(pols, n_pols, n).unwrap();
        let (group_elements, mp) = tree.get_group_proof(idx).unwrap();
        let root = tree.root();

        let re = root.as_elements();
        let expected = vec![
            FGL::from(10952823080416094333u64),
            FGL::from(14127307315435918656u64),
            FGL::from(18155557507084305090u64),
            FGL::from(4650815682547343351u64),
        ];
        assert_eq!(expected, re);

        assert!(tree.verify_group_proof(&root, &mp, idx, &group_elements).unwrap());
    }

    #[test]
    fn test_merklehash_gl_big() {
        let n = 1 << 16;
        let idx = 32;
        let n_pols = 50;
        let mut pols: Vec<FGL> = vec![FGL::ZERO; n_pols * n];
        for i in 0..n {
            for j in 0..n_pols {
                pols[i * n_pols + j] = FGL::from((i + j * 1000) as u64);
            }
        }

        let mut tree = MerkleTreeGL::new();
        tree.merkelize(pols, n_pols, n).unwrap();
        let (group_elements, mp) = tree.get_group_proof(idx).unwrap();
        let root = tree.root();
        assert!(tree.verify_group_proof(&root, &mp, idx, &group_elements).unwrap());
    }

    #[test]
    fn test_merkle_tree_gl_serialize_and_deserialize() {
        let data = MerkleTreeGL::new();
        let serialized = serde_json::to_string(&data).unwrap();
        let expect: MerkleTreeGL = serde_json::from_str(&serialized).unwrap();
        assert_eq!(data, expect);
    }
}
