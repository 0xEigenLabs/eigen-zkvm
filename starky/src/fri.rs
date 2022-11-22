use winter_math::{polynom, log2};
use crate::f3g::F3G;
use crate::merklehash_bn128::MerkleTree;
use crate::transcript_bn128::TranscriptBN128;

type struct FRI {
    in_nbits: i32,
    max_deg_nbits: i32,
    nqueries: i32,
    merklehash: MerkleTree,
    steps: HashMap<String, i32>,
};

impl FRI {

    pub fn prove(&mut self, transcript: &TranscriptBN128, pol, query_pol) {

    }
}
