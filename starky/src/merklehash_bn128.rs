use crate::digest_bn128::ElementDigest;
use crate::errors::Result;
use crate::linearhash_bn128::LinearHashBN128;
use crate::poseidon_bn128::Fr;
use winter_crypto::Hasher;
use winter_crypto::MerkleTree;
use winter_math::fields::f64::BaseElement;
pub struct MerkelHash();

impl MerkelHash {
    pub fn merkelize(columns: &Vec<Vec<BaseElement>>) -> Result<()> {
        let leaf_hash = LinearHashBN128::new();

        // TODO: hash leaves parallelly
        let mut leaves: Vec<crate::ElementDigest> = vec![];
        for col in columns.iter() {
            let digest: Fr = leaf_hash.hash_element_matrix(&vec![col.to_vec()]).unwrap();
            let digest: ElementDigest = ElementDigest::from(&digest);
            let node: [ElementDigest; 2] = [digest.clone(), digest];
            leaves.push(LinearHashBN128::merge(&node))
        }
        let tree = MerkleTree::<LinearHashBN128>::new(leaves);

        Ok(())
    }
}
