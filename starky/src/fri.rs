use winter_math::{polynom, log2};
use crate::f3g::F3G;
use crate::merklehash_bn128::MerkleTree;
use crate::transcript_bn128::TranscriptBN128;

type struct FRI {
    in_nbits: usize,
    max_deg_nbits: usize,
    nqueries: usize,
    merklehash: MerkleTree,
    steps: HashMap<String, usize>,
};

impl FRI {

    pub fn prove(&mut self, transcript: &TranscriptBN128, pol, query_pol) {

    }
}
