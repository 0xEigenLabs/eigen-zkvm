use crate::digest_bn128::ElementDigest;
use crate::errors::Result;
use crate::linearhash_bn128::LinearHashBN128;
use winter_crypto::MerkleTree;
use winter_math::fields::f64::BaseElement;

pub struct MerkelHash();

impl MerkelHash {
    pub fn merkelize(columns: &Vec<Vec<BaseElement>>) -> Result<()> {
        let leave_hash = LinearHashBN128::new();

        //let tree = MerkleTree::new(vec![]);
        Ok(())
    }
}
