#![allow(dead_code)]
use crate::constant::{get_max_workers, MAX_OPS_PER_THREAD, MIN_OPS_PER_THREAD};
use crate::digest::ElementDigest;
use crate::errors::{EigenError, Result};
use crate::f3g::F3G;
use crate::linearhash::LinearHash;
use crate::poseidon_opt::Poseidon;
use crate::traits::MerkleTree;
use plonky::field_gl::Fr as FGL;
use rayon::prelude::*;
use std::time::Instant;

#[derive(Default)]
pub struct MerkleTreeGL {
    pub elements: Vec<FGL>,
    pub width: usize,
    pub height: usize,
    pub nodes: Vec<ElementDigest<4>>,
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
                || Vec::<ElementDigest<4>>::new(),
                |mut a: Vec<ElementDigest<4>>, mut b: Vec<ElementDigest<4>>| {
                    a.append(&mut b);
                    a
                },
            );

        let out = &mut self.nodes[p_out..(p_out + n_ops)];
        out.iter_mut()
            .zip(nodes)
            .for_each(|(nout, nin)| *nout = nin);
        Ok(())
    }

    fn do_merklize_level(
        &self,
        buff_in: &[ElementDigest<4>],
        _st_i: usize,
        _st_n: usize,
    ) -> Result<Vec<ElementDigest<4>>> {
        log::debug!(
            "merklizing GL hash start.... {}/{}, buff size {}",
            _st_i,
            _st_n,
            buff_in.len()
        );
        let n_ops = buff_in.len() / 2;
        let mut buff_out64: Vec<ElementDigest<4>> = vec![ElementDigest::<4>::default(); n_ops];
        buff_out64
            .iter_mut()
            .zip((0..n_ops).into_iter())
            .for_each(|(out, i)| {
                let mut two = [FGL::ZERO; 8];
                let one: &[FGL] = buff_in[i * 2].as_elements();
                two[0..4].copy_from_slice(&one);
                let one: &[FGL] = buff_in[i * 2 + 1].as_elements();
                two[4..8].copy_from_slice(&one);
                *out = self.h.hash(&two, 0).unwrap();
            });
        Ok(buff_out64)
    }

    fn merkle_calculate_root_from_proof(
        &self,
        mp: &Vec<Vec<FGL>>,
        idx: usize,
        value: &ElementDigest<4>,
        offset: usize,
    ) -> Result<ElementDigest<4>> {
        if mp.len() == offset {
            return Ok(value.clone());
        }
        let cur_idx = idx & 1;
        let next_idx = idx / 2;
        let init = [FGL::ZERO; 4];
        let next_value;
        let mut inhash = vec![FGL::ZERO; 8];
        if cur_idx == 0 {
            let one = value.as_elements();
            inhash[0..4].copy_from_slice(&one);
            for i in 0..4 {
                inhash[4 + i] = mp[offset][i];
            }
        } else {
            for i in 0..4 {
                inhash[i] = mp[offset][i];
            }
            let one = value.as_elements();
            inhash[4..8].copy_from_slice(&one);
        }
        let next = self.poseidon.hash(&inhash, &init, 4)?;
        next_value = ElementDigest::<4>::new(next.try_into().unwrap());
        self.merkle_calculate_root_from_proof(mp, next_idx, &next_value, offset + 1)
    }

    fn calculate_root_from_group_proof(
        &self,
        mp: &Vec<Vec<FGL>>,
        idx: usize,
        vals: &Vec<FGL>,
    ) -> Result<ElementDigest<4>> {
        let h = self.h.hash(vals, 0)?;
        self.merkle_calculate_root_from_proof(mp, idx, &h, 0)
    }
}

impl MerkleTree for MerkleTreeGL {
    type BaseField = FGL;

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
        return self.elements.len();
    }

    fn to_f3g(&self, p_be: &mut Vec<F3G>) {
        assert_eq!(p_be.len(), self.elements.len());
        p_be.par_iter_mut()
            .zip(&self.elements)
            .for_each(|(be_out, f3g_in)| {
                *be_out = F3G::from(*f3g_in);
            });
    }

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

        let mut nodes = vec![ElementDigest::<4>::default(); get_n_nodes(height)];
        let now = Instant::now();
        if buff.len() > 0 {
            nodes
                .par_chunks_mut(n_per_thread_f)
                .zip(buff.par_chunks(n_per_thread_f * width))
                .for_each(|(out, bb)| {
                    let cur_n = bb.len() / width;
                    out.iter_mut()
                        .zip((0..cur_n).into_iter())
                        .for_each(|(row_out, j)| {
                            let batch = &bb[(j * width)..((j + 1) * width)];
                            *row_out = self.h.hash(batch, 0).unwrap();
                        });
                });
        }
        log::info!("linearhash time cost: {}", now.elapsed().as_secs_f64());

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
            log::info!(
                "merklize_level {} time cost: {}",
                next_n64,
                now.elapsed().as_secs_f64()
            );
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
            return Err(EigenError::MerkleTreeError(
                "access invalid node".to_string(),
            ));
        }

        let mut v = vec![FGL::ZERO; self.width];
        for i in 0..self.width {
            v[i] = self.get_element(idx, i);
        }
        let mp = self.merkle_gen_merkle_proof(idx, 0, self.height);
        Ok((v, mp))
    }

    fn eq_root(&self, r1: &ElementDigest<4>, r2: &ElementDigest<4>) -> bool {
        r1 == r2
    }

    fn verify_group_proof(
        &self,
        root: &ElementDigest<4>,
        mp: &Vec<Vec<FGL>>,
        idx: usize,
        group_elements: &Vec<FGL>,
    ) -> Result<bool> {
        let c_root = self.calculate_root_from_group_proof(mp, idx, group_elements)?;
        Ok(self.eq_root(root, &c_root))
    }

    fn root(&self) -> ElementDigest<4> {
        self.nodes[self.nodes.len() - 1]
    }
}

#[cfg(test)]
mod tests {
    use crate::merklehash::MerkleTreeGL;
    use crate::traits::MerkleTree;
    use plonky::field_gl::Fr as FGL;

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

        let mut tree = MerkleTreeGL::new();
        tree.merkelize(cols, n_pols, n).unwrap();
        let (v, mp) = tree.get_group_proof(idx).unwrap();
        let root = tree.root();
        let re = root.as_elements();
        let expected = vec![
            FGL::from(11508832812350783315u64),
            FGL::from(5044133147279090978u64),
            FGL::from(6335412741057168694u64),
            FGL::from(12530816673814004438u64),
        ];
        assert_eq!(expected, re);

        assert_eq!(tree.verify_group_proof(&root, &mp, idx, &v).unwrap(), true);
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
        assert_eq!(
            tree.verify_group_proof(&root, &mp, idx, &group_elements)
                .unwrap(),
            true
        );
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

        assert_eq!(
            tree.verify_group_proof(&root, &mp, idx, &group_elements)
                .unwrap(),
            true
        );
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
        assert_eq!(
            tree.verify_group_proof(&root, &mp, idx, &group_elements)
                .unwrap(),
            true
        );
    }
    //TODO save and restore to file
}
