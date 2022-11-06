use crate::constant::{get_max_workers, max_ops_per_thread, min_ops_per_thread};
use crate::digest_bn128::ElementDigest;
use crate::errors::{EigenError, Result};
use crate::linearhash_bn128::LinearHashBN128;
use crate::poseidon_bn128::{Fr, Poseidon};
use winter_crypto::Hasher;
use winter_crypto::MerkleTree;
use winter_math::fields::f64::BaseElement;
use winter_math::StarkField;

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
        println!("cur_n {} {}", i, i + cur_n);
        for j in 0..cur_n {
            batch.append(&mut columns[i + j].clone());
            println!("batch");
            let ccc: Vec<u32> = batch
                .iter()
                .map(|e| {
                    println!("b: {}", e);
                    1u32
                })
                .collect();

            // TODO: parallel hash
            let node = leaves_hash.hash_element_array(&batch)?;
            //let node = ElementDigest::new(LinearHashBN128::to_bn128_mont(&node.0));

            println!("height {} ", j);
            let ddd: Vec<_> = node
                .0
                .iter()
                .map(|e| {
                    print!("n: {:?} ", e.as_int());
                    1u32
                })
                .collect();
            println!("");
            leaves.push(node);
            batch = vec![];
        }
    }

    println!("leaves size {}", leaves.len());
    // merklize level
    let tree = MerkleTree::<Poseidon>::new(leaves)
        .map_err(|e| EigenError::MerkleTreeError(e.to_string()))?;

    Ok(tree)
}

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

        println!("root : {:?}", root);
    }
}
