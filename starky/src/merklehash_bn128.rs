use crate::constant::{get_max_workers, MAX_OPS_PER_THREAD, MIN_OPS_PER_THREAD};
use crate::digest_bn128::ElementDigest;
use crate::errors::{EigenError, Result};
use crate::linearhash_bn128::LinearHashBN128;
use crate::poseidon_bn128::{Fr, Poseidon};
use winter_crypto::Hasher;
use winter_math::fields::f64::BaseElement;
use winter_math::StarkField;

pub struct MerkleTree {
    pub elements: Vec<Vec<BaseElement>>,
    pub nodes: Vec<ElementDigest>,
    pub h: LinearHashBN128,
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

pub fn merkelize(columns: &Vec<Vec<BaseElement>>) -> Result<MerkleTree> {
    let leaves_hash = LinearHashBN128::new();

    let mut leaves: Vec<crate::ElementDigest> = vec![];
    let mut batch: Vec<BaseElement> = vec![];

    let height = columns.len();
    let width = columns[0].len();
    let max_workers = get_max_workers();

    let mut n_per_thread_f = (height - 1) / max_workers + 1;
    let min_pt = MIN_OPS_PER_THREAD / ((width - 1) / (3 * 16) + 1);
    if n_per_thread_f < min_pt {
        n_per_thread_f = min_pt;
    }
    if n_per_thread_f > MAX_OPS_PER_THREAD {
        n_per_thread_f = MAX_OPS_PER_THREAD;
    }

    println!("n_per_thread_f: {}, height {}", n_per_thread_f, height);
    for i in (0..height).step_by(n_per_thread_f) {
        let cur_n = std::cmp::min(n_per_thread_f, height - i);
        // get elements from row i to i + cur_n
        println!("cur_n {} {}", i, i + cur_n);
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

    println!("leaves size {}", leaves.len());
    // merklize level
    let mut tree = MerkleTree {
        elements: columns.clone(),
        nodes: vec![ElementDigest::default(); get_n_nodes(columns.len()) * 4],
        h: leaves_hash,
    };

    let mut n256: usize = height;
    let mut next_n256: usize = (n256 - 1) / 16 + 1;
    let mut p_in: usize = 0;
    let mut p_out: usize = p_in + next_n256 * 16;
    while n256 > 1 {
        merklize_level(&mut tree, p_in, next_n256, p_out);
        n256 = next_n256;
        next_n256 = (n256 - 1) / 16 + 1;
        p_in = p_out;
        p_out = p_in + next_n256 * 16;
    }

    Ok(tree)
}

pub fn merklize_level(
    tree: &mut MerkleTree,
    p_in: usize,
    n_ops: usize,
    p_out: usize,
) -> Result<()> {
    let n_ops_per_thread = (n_ops - 1) / get_max_workers() + 1;
    if n_ops_per_thread < MIN_OPS_PER_THREAD {
        n_ops_per_thread = MIN_OPS_PER_THREAD;
    }

    for i in (0..n_ops).step_by(n_ops_per_thread) {
        let cur_n_ops = std::cmp::min(n_ops_per_thread, n_ops - i);
        let bb = &tree.nodes[(p_in / 8 + i)..(p_in / 8 + (i + cur_n_ops))];
        let res = do_merklize_level(tree, bb, i, n_ops)?;
        tree.nodes[p_out / 8 + i * n_ops_per_thread * 4] = res;
    }
    Ok(())
}

fn do_merklize_level(
    tree: &MerkleTree,
    buff_in: &[ElementDigest],
    st_i: usize,
    st_n: usize,
) -> Result<Vec<ElementDigest>> {
    println!("merklizing bn128 hash start.... {}/{}", st_i, st_n);
    let n_ops = buff_in.len() / (4 * 16);
    let mut buff_out64: Vec<ElementDigest> = vec![];
    for i in 0..n_ops {
        let digest: Fr = Fr::zero();
        buff_out64.push(tree.h.inner_hash_block(buff_in, &digest)?);
    }
    Ok(buff_out64)
}

pub fn get_element(tree: &MerkleTree, idx: usize, sub_idx: usize) -> BaseElement {
    tree.elements[sub_idx][idx]
}

/*
pub fn get_group_proof(tree: &MerkleTree, idx) -> Result <()> {
    if idx < 0 || idx >= tree.columns.len() {
        return EigenError::MerkleTreeError("access invalid node".to_string())
    }

    let v =
}
*/

#[cfg(test)]
mod tests {
    use crate::merklehash_bn128::merkelize;
    use crate::poseidon_bn128::Fr;
    use crate::traits::FieldMapping;
    use crate::ElementDigest;
    use winter_math::{fields::f64::BaseElement, FieldElement};

    #[test]
    fn test_merklehash() {
        let n_pols = 13;
        let n = 8;

        let mut cols: Vec<Vec<BaseElement>> = vec![Vec::new(); n];
        for i in 0..n {
            cols[i] = vec![BaseElement::ZERO; n_pols];
            for j in 0..n_pols {
                cols[i][j] = BaseElement::from((i + j * 1000) as u32);
            }
        }

        let tree = merkelize(&cols).unwrap();
        let root = tree.root();

        let bn: Fr = (*root).into();
        let bn_mont = ElementDigest::to_montgomery(&bn);

        println!("root : {:?} {:?}", bn.to_string(), bn_mont.to_string());
    }
}
