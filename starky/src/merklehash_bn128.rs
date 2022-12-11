#![allow(dead_code)]
use crate::constant::{get_max_workers, MAX_OPS_PER_THREAD, MIN_OPS_PER_THREAD};
use crate::digest_bn128::ElementDigest;
use crate::errors::{EigenError, Result};
use crate::f3g::F3G;
use crate::field_bn128::Fr;
use crate::linearhash_bn128::LinearHashBN128;
use crate::poseidon_bn128_opt::Poseidon;
use ff::Field;
use rayon::prelude::*;
use std::time::Instant;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;

#[derive(Default)]
pub struct MerkleTree {
    pub elements: Vec<BaseElement>,
    pub width: usize,
    pub height: usize,
    pub nodes: Vec<ElementDigest>,
    h: LinearHashBN128,
    poseidon: Poseidon,
}

fn get_n_nodes(n_: usize) -> usize {
    let mut n = n_;
    let mut next_n = (n - 1) / 16 + 1;
    let mut acc = next_n * 16;
    while n > 1 {
        n = next_n;
        next_n = (n - 1) / 16 + 1;
        if n > 1 {
            acc += next_n * 16;
        } else {
            acc += 1;
        }
    }
    acc
}

impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree {
            nodes: Vec::new(),
            elements: Vec::new(),
            h: LinearHashBN128::new(),
            width: 0,
            height: 0,
            poseidon: Poseidon::new(),
        }
    }

    pub fn to_f3g(&self, p_be: &mut Vec<F3G>) {
        assert_eq!(p_be.len(), self.elements.len());
        p_be.par_iter_mut()
            .zip(&self.elements)
            .for_each(|(be_out, f3g_in)| {
                *be_out = F3G::from(*f3g_in);
            });
    }

    pub fn merkelize(buff: Vec<BaseElement>, width: usize, height: usize) -> Result<Self> {
        let leaves_hash = LinearHashBN128::new();

        //println!("width {}, height {}, {:?}", width, height, buff);
        let max_workers = get_max_workers();

        let mut n_per_thread_f = (height - 1) / max_workers + 1;
        let mut min_pt = 0;
        if width > 1 {
            min_pt = MIN_OPS_PER_THREAD / ((width - 1) / (3 * 16) + 1);
        }
        if n_per_thread_f < min_pt {
            n_per_thread_f = min_pt;
        }
        if n_per_thread_f > MAX_OPS_PER_THREAD {
            n_per_thread_f = MAX_OPS_PER_THREAD;
        }
        let mut nodes = vec![ElementDigest::default(); get_n_nodes(height)];
        let now = Instant::now();
        //println!("n_per_thread_f: {}, height {}", n_per_thread_f, height);
        if buff.len() > 0 {
            nodes
                .par_chunks_mut(n_per_thread_f)
                .zip(buff.par_chunks(n_per_thread_f * width))
                .for_each(|(out, bb)| {
                    let cur_n = bb.len() / width;
                    //println!("linearhash pols i {}, cur_n {}", i, cur_n);
                    out.iter_mut()
                        .zip((0..cur_n).into_iter())
                        .for_each(|(row_out, j)| {
                            let batch = &bb[(j * width)..((j + 1) * width)];
                            *row_out = leaves_hash.hash_element_array(batch).unwrap();
                        });
                });
        }
        println!("linearhash time cost: {}", now.elapsed().as_secs_f64());

        // merklize level
        let mut tree = MerkleTree {
            nodes: nodes,
            elements: buff,
            h: leaves_hash,
            width: width,
            height: height,
            poseidon: Poseidon::new(),
        };

        //println!("len {}, height {}, leave size {}", tree.nodes.len(), height, leaves.len());

        let mut n256: usize = height;
        let mut next_n256: usize = (n256 - 1) / 16 + 1;
        let mut p_in: usize = 0;
        let mut p_out: usize = p_in + next_n256 * 16;
        while n256 > 1 {
            //println!("p_in {}, next_n256 {}, p_out {}", p_in, next_n256, p_out);
            let now = Instant::now();
            tree.merklize_level(p_in, next_n256, p_out)?;
            println!(
                "merklize_level {} time cost: {}",
                next_n256,
                now.elapsed().as_secs_f64()
            );
            n256 = next_n256;
            next_n256 = (n256 - 1) / 16 + 1;
            p_in = p_out;
            p_out = p_in + next_n256 * 16;
        }

        Ok(tree)
    }

    pub fn merklize_level(&mut self, p_in: usize, n_ops: usize, p_out: usize) -> Result<()> {
        let mut n_ops_per_thread = (n_ops - 1) / get_max_workers() + 1;
        if n_ops_per_thread < MIN_OPS_PER_THREAD {
            n_ops_per_thread = MIN_OPS_PER_THREAD;
        }

        let buff = &self.nodes[p_in..(p_in + n_ops * 16)];
        //println!("merklize level: hash {} to {}", p_in, p_out);
        let nodes = buff
            .par_chunks(16 * n_ops_per_thread)
            .enumerate()
            .map(|(i, bb)| self.do_merklize_level(bb, i, n_ops).unwrap())
            .reduce(
                || Vec::<ElementDigest>::new(),
                |mut a: Vec<ElementDigest>, mut b: Vec<ElementDigest>| {
                    a.append(&mut b);
                    a
                },
            );

        //println!("merklize level: copy {} to {}", p_in, p_out);
        let out = &mut self.nodes[p_out..(p_out + n_ops)];
        out.iter_mut()
            .zip(nodes)
            .for_each(|(nout, nin)| *nout = nin);
        Ok(())
    }

    fn do_merklize_level(
        &self,
        buff_in: &[ElementDigest],
        _st_i: usize,
        _st_n: usize,
    ) -> Result<Vec<ElementDigest>> {
        println!(
            "merklizing bn128 hash start.... {}/{}, buff size {}",
            _st_i,
            _st_n,
            buff_in.len()
        );
        let n_ops = buff_in.len() / 16;
        let mut buff_out64: Vec<ElementDigest> = vec![ElementDigest::default(); n_ops];
        buff_out64
            .iter_mut()
            .zip((0..n_ops).into_iter())
            .for_each(|(out, i)| {
                *out = self
                    .h
                    .hash_node(&buff_in[(i * 16)..(i * 16 + 16)], &Fr::zero())
                    .unwrap();
            });
        Ok(buff_out64)
    }

    pub fn get_element(&self, idx: usize, sub_idx: usize) -> BaseElement {
        self.elements[self.width * idx + sub_idx]
    }

    fn merkle_gen_merkle_proof(&self, idx: usize, offset: usize, n: usize) -> Vec<Vec<Fr>> {
        if n <= 1 {
            return vec![];
        }
        let next_idx = idx >> 4;
        let si = idx & 0xFFFFFFF0;
        let mut sibs: Vec<Fr> = vec![];

        for i in 0..16 {
            let sib = self.nodes[offset + (si + i)].into();
            sibs.push(sib);
        }

        let next_n = (n - 1) / 16 + 1;
        let mut result = vec![sibs];
        result.append(&mut self.merkle_gen_merkle_proof(next_idx, offset + next_n * 16, next_n));
        result
    }

    pub fn get_group_proof(&self, idx: usize) -> Result<(Vec<BaseElement>, Vec<Vec<Fr>>)> {
        if idx >= self.height {
            return Err(EigenError::MerkleTreeError(
                "access invalid node".to_string(),
            ));
        }

        let mut v = vec![BaseElement::ZERO; self.width];
        for i in 0..self.width {
            v[i] = self.get_element(idx, i);
        }
        let mp = self.merkle_gen_merkle_proof(idx, 0, self.height);

        Ok((v, mp))
    }

    fn merkle_calculate_root_from_proof(
        &self,
        mp: &Vec<Vec<Fr>>,
        idx: usize,
        value: &ElementDigest,
        offset: usize,
    ) -> Result<ElementDigest> {
        if mp.len() == offset {
            return Ok(value.clone());
        }
        //let cur_idx = idx & 0xF;
        let next_idx = idx >> 4;
        let mut vals: Vec<Fr> = vec![];
        for i in 0..16 {
            vals.push(mp[offset][i]);
        }
        let init = Fr::zero();
        let next_value = self.poseidon.hash(&vals, &init)?;
        let next_value = ElementDigest::from(&next_value);
        self.merkle_calculate_root_from_proof(mp, next_idx, &next_value, offset + 1)
    }

    pub fn calculate_root_from_group_proof(
        &self,
        mp: &Vec<Vec<Fr>>,
        idx: usize,
        vals: &Vec<BaseElement>,
    ) -> Result<ElementDigest> {
        let h = self.h.hash_element_matrix(&vec![vals.to_vec()])?;
        self.merkle_calculate_root_from_proof(mp, idx, &ElementDigest::from(&h), 0)
    }

    pub fn eq_root(&self, r1: &ElementDigest, r2: &ElementDigest) -> bool {
        r1 == r2
    }

    pub fn verify_group_proof(
        &self,
        root: &ElementDigest,
        mp: &Vec<Vec<Fr>>,
        idx: usize,
        group_elements: &Vec<BaseElement>,
    ) -> Result<bool> {
        let c_root = self.calculate_root_from_group_proof(mp, idx, group_elements)?;
        Ok(self.eq_root(root, &c_root))
    }

    pub fn root(&self) -> ElementDigest {
        self.nodes[self.nodes.len() - 1]
    }
}

#[cfg(test)]
mod tests {
    use crate::field_bn128::Fr;
    use crate::merklehash_bn128::MerkleTree;
    use ff::PrimeField;
    use winter_math::fields::f64::BaseElement;
    use winter_math::FieldElement;

    #[test]
    fn test_merklehash() {
        // https://github.com/0xPolygonHermez/pil-stark/blob/main/test/merklehash.bn128.test.js#L16
        let n = 256;
        let idx = 3;
        let n_pols = 9;

        let mut cols: Vec<BaseElement> = vec![BaseElement::ZERO; n_pols * n];
        for i in 0..n {
            for j in 0..n_pols {
                cols[i * n_pols + j] = BaseElement::from((i + j * 1000) as u64);
            }
        }

        let tree = MerkleTree::merkelize(cols, n_pols, n).unwrap();
        let root: Fr = tree.root().into();
        assert_eq!(
            root,
            Fr::from_str(
                "2052732265221205192391066587135329070685482706470940527184785165917406935559"
            )
            .unwrap()
        );

        let (v, mp) = tree.get_group_proof(idx).unwrap();
        let root = tree.root();
        assert_eq!(tree.verify_group_proof(&root, &mp, idx, &v).unwrap(), true);
    }

    #[test]
    fn test_merklehash_small() {
        let n = 256;
        let idx = 3;
        let n_pols = 9;
        let mut pols: Vec<BaseElement> = vec![BaseElement::ZERO; n_pols * n];
        for i in 0..n {
            for j in 0..n_pols {
                pols[i * n_pols + j] = BaseElement::from((i + j * 1000) as u32);
            }
        }

        let tree = MerkleTree::merkelize(pols, n_pols, n).unwrap();
        let (group_elements, mp) = tree.get_group_proof(idx).unwrap();
        let root = tree.root();
        assert_eq!(
            tree.verify_group_proof(&root, &mp, idx, &group_elements)
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_merklehash_not_power_of_2() {
        let n = 33;
        let idx = 32;
        let n_pols = 6;
        let mut pols: Vec<BaseElement> = vec![BaseElement::ZERO; n_pols * n];
        for i in 0..n {
            for j in 0..n_pols {
                pols[i * n_pols + j] = BaseElement::from((i + j * 1000) as u32);
            }
        }

        let tree = MerkleTree::merkelize(pols, n_pols, n).unwrap();
        let (group_elements, mp) = tree.get_group_proof(idx).unwrap();
        let root = tree.root();
        assert_eq!(
            tree.verify_group_proof(&root, &mp, idx, &group_elements)
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_merklehash_big() {
        let n = 1 << 16;
        let idx = 32;
        let n_pols = 50;
        let mut pols: Vec<BaseElement> = vec![BaseElement::ZERO; n_pols * n];
        for i in 0..n {
            for j in 0..n_pols {
                pols[i * n_pols + j] = BaseElement::from((i + j * 1000) as u32);
            }
        }

        let tree = MerkleTree::merkelize(pols, n_pols, n).unwrap();
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
