use crate::constant::{get_max_workers, MAX_OPS_PER_THREAD, MIN_OPS_PER_THREAD};
use crate::digest_bn128::ElementDigest;
use crate::errors::{EigenError, Result};
use crate::f3g::F3G;
use crate::linearhash_bn128::LinearHashBN128;
use crate::poseidon_bn128::{Fr, Poseidon};
use ff::Field;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;

#[derive(Default)]
pub struct MerkleTree {
    pub elements: Vec<Vec<BaseElement>>,
    pub width: usize,
    pub height: usize,
    pub nodes: Vec<ElementDigest>,
    h: LinearHashBN128,
    poseidon: Poseidon,
}

impl MerkleTree {
    pub fn write_buff(&self) -> Vec<F3G> {
        let mut buff: Vec<F3G> = vec![];
        for i in 0..self.width {
            for j in 0..self.height {
                buff.push(F3G::from(self.elements[j][i]));
            }
        }
        buff
    }
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

    pub fn merkelize(columns: Vec<Vec<BaseElement>>, width: usize, height: usize) -> Result<Self> {
        let leaves_hash = LinearHashBN128::new();

        let mut leaves: Vec<crate::ElementDigest> = vec![];
        let mut batch: Vec<BaseElement> = vec![];

        //println!("width {}, height {}, {:?}", width, height, columns);
        let max_workers = get_max_workers();

        let mut n_per_thread_f = (height - 1) / max_workers + 1;
        let min_pt = MIN_OPS_PER_THREAD / ((width - 1) / (3 * 16) + 1);
        if n_per_thread_f < min_pt {
            n_per_thread_f = min_pt;
        }
        if n_per_thread_f > MAX_OPS_PER_THREAD {
            n_per_thread_f = MAX_OPS_PER_THREAD;
        }

        //println!("n_per_thread_f: {}, height {}", n_per_thread_f, height);
        for i in (0..height).step_by(n_per_thread_f) {
            let cur_n = std::cmp::min(n_per_thread_f, height - i);
            // get elements from row i to i + cur_n
            //println!("cur_n {} {}", i, i + cur_n);
            for j in 0..cur_n {
                batch.append(&mut columns[i + j].clone());
                /*
                println!("batch");
                let ccc: Vec<u32> = batch
                    .iter()
                    .map(|e| {
                        println!("b: {}", e);
                        1u32
                    })
                    .collect();
                */

                // TODO: parallel hash
                let node = leaves_hash.hash_element_array(&batch)?;

                /*
                let ddd: Vec<_> = node
                    .0
                    .iter()
                    .map(|e| {
                        print!("hased result: {:?} ", e.as_int());
                        1u32
                    })
                    .collect();
                println!("");
                */
                leaves.push(node);
                batch = vec![];
            }
        }

        //println!("leaves size {}", leaves.len());
        // merklize level
        let mut tree = MerkleTree {
            nodes: vec![ElementDigest::default(); get_n_nodes(height)],
            elements: columns,
            h: leaves_hash,
            width: width,
            height: height,
            poseidon: Poseidon::new(),
        };

        // set leaves
        for (i, leaf) in leaves.iter().enumerate() {
            tree.nodes[i] = *leaf;
        }

        let mut n256: usize = height;
        let mut next_n256: usize = (n256 - 1) / 16 + 1;
        let mut p_in: usize = 0;
        let mut p_out: usize = p_in + next_n256 * 16;
        while n256 > 1 {
            //println!("p_in {}, next_n256 {}, p_out {}", p_in, next_n256, p_out);
            tree.merklize_level(p_in, next_n256, p_out)?;
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

        //println!("merkelize_level ops {} n_pt {}", n_ops, n_ops_per_thread);
        for i in (0..n_ops).step_by(n_ops_per_thread) {
            let cur_n_ops = std::cmp::min(n_ops_per_thread, n_ops - i);
            //println!("p_in={}, cur_n_ops={}", p_in, cur_n_ops);
            let bb = &self.nodes[(p_in + i * 16)..(p_in + (i + cur_n_ops) * 16)];
            /*
            println!(
                ">>>  handle {} to {}",
                (p_in + i * 16),
                p_in + (i + cur_n_ops) * 16
            );
            */
            let res = self.do_merklize_level(bb, i, n_ops)?;
            for (j, v) in res.iter().enumerate() {
                let idx = p_out + i * n_ops_per_thread + j;
                //println!("set {}, {:?}", idx, self.nodes[idx]);
                self.nodes[idx] = *v;

                /*println!("to: {:?}, which is ", self.nodes[idx]);
                let ddd: Vec<_> = self.nodes[idx]
                    .0
                    .iter()
                    .map(|e| {
                        print!("hased result: {:?} ", e.as_int());
                        1u32
                    })
                    .collect();
                */
            }
        }
        Ok(())
    }

    fn do_merklize_level(
        &self,
        buff_in: &[ElementDigest],
        st_i: usize,
        st_n: usize,
    ) -> Result<Vec<ElementDigest>> {
        println!(
            "merklizing bn128 hash start.... {}/{}, buff size {}",
            st_i,
            st_n,
            buff_in.len()
        );
        let n_ops = buff_in.len() / 16;
        let mut buff_out64: Vec<ElementDigest> = vec![];
        for i in 0..n_ops {
            let digest: Fr = Fr::zero();
            buff_out64.push(
                self.h
                    .inner_hash_digest(&buff_in[(i * 16)..(i * 16 + 16)], &digest)?,
            );
        }
        Ok(buff_out64)
    }

    pub fn get_element(&self, idx: usize, sub_idx: usize) -> BaseElement {
        self.elements[idx][sub_idx]
    }

    fn merkle_gen_merkle_proof(&self, idx: usize, offset: usize, n: usize) -> Vec<Vec<Fr>> {
        if n <= 1 {
            return vec![];
        }
        let next_idx = idx >> 4;
        let si = idx & 0xFFFFFFF0;
        let mut sibs: Vec<Fr> = vec![];

        for i in 0..16 {
            let buff8 = self.nodes[offset + (si + i)].into();
            sibs.push(buff8);
        }

        let next_n = (n - 1) / 16 + 1;

        let mut result = vec![sibs];
        result.append(&mut self.merkle_gen_merkle_proof(next_idx, offset + next_n * 16, next_n));
        result
    }

    pub fn get_group_proof(&self, idx: usize) -> Result<(Vec<BaseElement>, Vec<Vec<Fr>>)> {
        if idx >= self.width {
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
        let cur_idx = idx & 0xF;
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
    use crate::merklehash_bn128::MerkleTree;
    use crate::poseidon_bn128::Fr;
    use crate::traits::FieldMapping;
    use crate::ElementDigest;
    use ff::PrimeField;
    use winter_math::{fields::f64::BaseElement, FieldElement, StarkField};

    #[test]
    fn test_merklehash() {
        // https://github.com/0xPolygonHermez/pil-stark/blob/main/test/merklehash.bn128.test.js#L16
        let n_pols = 13;
        let n = 256;
        let idx = 3;

        let mut cols: Vec<Vec<BaseElement>> = vec![Vec::new(); n];
        for i in 0..n {
            cols[i] = vec![BaseElement::ZERO; n_pols];
            for j in 0..n_pols {
                cols[i][j] = BaseElement::from((i + j * 1000) as u32);
            }
        }

        let tree = MerkleTree::merkelize(cols, n_pols, n).unwrap();

        let (v, mp) = tree.get_group_proof(idx).unwrap();
        println!("get_group_proof: {},\n v = ", idx);
        v.iter()
            .map(|e| println!("{:?}", e.as_int()))
            .collect::<Vec<()>>();
        println!("mp = ");
        for (i, p) in mp.iter().enumerate() {
            println!("next {}", i);
            p.iter()
                .map(|e| println!("{:?}", e.to_string()))
                .collect::<Vec<()>>();
        }

        let root: Fr = tree.root().into();
        println!("root {:?}", root);
        assert_eq!(
            root,
            Fr::from_str(
                "8005252974590666002771739711749534229428809787999120161010044101718518171945"
            )
            .unwrap()
        );

        let root = tree.root();
        assert_eq!(tree.verify_group_proof(&root, &mp, idx, &v).unwrap(), true);
    }
}
