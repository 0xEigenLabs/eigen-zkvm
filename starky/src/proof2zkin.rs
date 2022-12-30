use crate::stark_gen::StarkProof;
use crate::traits::MerkleTree;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ZKIn {}

pub fn proof2zkin<M: MerkleTree>(proof: &StarkProof<M>) {}
