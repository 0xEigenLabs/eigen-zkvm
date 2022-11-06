use crate::constant::{get_max_workers, max_ops_per_thread, min_ops_per_thread};
use crate::digest_bn128::ElementDigest;
use crate::errors::Result;
use crate::linearhash_bn128::LinearHashBN128;
use crate::poseidon_bn128::{Fr, Poseidon};
use winter_crypto::Hasher;
use winter_crypto::MerkleTree;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;

pub fn merkelize(columns: &Vec<Vec<BaseElement>>) -> Result<MerkleTree<Poseidon>> {
    let leaves_hash = LinearHashBN128::new();

    let mut leaves: Vec<crate::ElementDigest> = vec![];
    let mut batch: Vec<BaseElement> = vec![];

    let height = columns.len();
    let width = columns[0].len();
    let max_workers = get_max_workers();

    let mut n_per_thread_f = (height - 1) / max_workers + 1;
    let min_pt = min_ops_per_thread / ((width - 1) / (3 * 16) + 1);
    if n_per_thread_f < min_pt {
        n_per_thread_f = min_pt;
    }
    if n_per_thread_f > max_ops_per_thread {
        n_per_thread_f = max_ops_per_thread;
    }

    println!("n_per_thread_f: {}, height {}", n_per_thread_f, height);
    for i in (0..height).step_by(n_per_thread_f) {
        let cur_n = std::cmp::min(n_per_thread_f, height - i);
        // get elements from row i to i + cur_n
        let mut batch: Vec<BaseElement> = vec![];
        for j in (0..cur_n) {
            batch.append(&mut columns[i + j].clone());
        }

        // TODO: parallel hash
        let mut node = LinearHashBN128::hash_node(&batch).unwrap();
        leaves.append(&mut node);
    }

    // merklize level
    let tree = MerkleTree::<Poseidon>::new(leaves).unwrap();

    Ok(tree)
}

#[cfg(test)]
mod tests {
    use winter_math::{fields::f64::BaseElement, FieldElement};

    use crate::merklehash_bn128::merkelize;
    #[test]
    fn test_merklehash() {
        let n_pols = 3;
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
        println!("root : {:?}", root);
    }
}
