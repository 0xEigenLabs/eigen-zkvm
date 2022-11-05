use crate::digest_bn128::ElementDigest;
use crate::errors::Result;
use crate::linearhash_bn128::LinearHashBN128;
use crate::poseidon_bn128::{Fr, Poseidon};
use winter_crypto::Hasher;
use winter_crypto::MerkleTree;
use winter_math::fields::f64::BaseElement;

pub fn merkelize(columns: &Vec<Vec<BaseElement>>) -> Result<MerkleTree<Poseidon>> {
    let leaf_hash = LinearHashBN128::new();

    // TODO: parallel hash
    let mut leaves: Vec<crate::ElementDigest> = vec![];
    for col in columns.iter() {
        // hash 32 elements each time
        let digest: Fr = leaf_hash.hash_element_matrix(&vec![col.to_vec()]).unwrap();
        let digest: ElementDigest = ElementDigest::from(&digest);
        let node: [ElementDigest; 2] = [digest.clone(), digest];
        leaves.push(LinearHashBN128::merge(&node))
    }

    // hash the nodes
    let tree = MerkleTree::<Poseidon>::new(leaves).unwrap();

    Ok(tree)
}


#[cfg(test)]
mod tests {
    use winter_math::{fields::f64::BaseElement, FieldElement};

    use crate::merklehash_bn128::merkelize;
    #[test]
    fn test_merklehash() {
        let nPols = 3;
        let n = 8;

        let mut cols: Vec<Vec<BaseElement>> = vec![Vec::new(); n];
        for i in 0..n {
            cols[i] = vec![BaseElement::ZERO; nPols]; 
            for j in 0..nPols {
                cols[i][j] = BaseElement::from((i + j * 1000) as u32);
            }
        }

        let tree = merkelize(&cols).unwrap();
        let root = tree.root();
        println!("root : {:?}", root);
    }
}
