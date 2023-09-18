use crate::errors::Result as EigenResult;
use crate::f3g::F3G;
use ff::PrimeField;
use plonky::field_gl::Fr as FGL;
use crate::ElementDigest;
use crate::field_bn128::{Fr, FrRepr};

pub trait MTNodeType {
    fn as_elements(&self) -> &[FGL];
    fn new(value: &[FGL]) -> Self;
    fn from_scalar<T: PrimeField>(e: &T) -> Self;
    fn as_bn128(self) -> crate::field_bn128::Fr;
}

pub trait MerkleTreeBase {
    type BaseField: Clone
    + Default
    + core::fmt::Debug
    + Into<crate::serializer::Input<Self::MTNode>>;
    type MTNode: Copy + std::fmt::Display + Clone + Default + MTNodeType + core::fmt::Debug;
    type PoseidonType: PoseidonTrait;

    fn merklize_level(&mut self, p_in: usize, n_ops: usize, p_out: usize) -> EigenResult<()>;

    fn do_merklize_level(
        &self,
        buff_in: &[Self::MTNode],
        _st_i: usize,
        _st_n: usize,
    ) -> EigenResult<Vec<Self::MTNode>>;

    fn merkle_gen_merkle_proof(&self, idx: usize, offset: usize, n: usize) -> Vec<Vec<Self::BaseField>>;

    fn merkle_calculate_root_from_proof(
        &self,
        mp: &Vec<Vec<Self::BaseField>>,
        idx: usize,
        value: &Self::MTNode,
        offset: usize,
    ) -> EigenResult<Self::MTNode>;

    fn calculate_root_from_group_proof(
        &self,
        mp: &Vec<Vec<Self::BaseField>>,
        idx: usize,
        vals: &Vec<FGL>,
    ) -> EigenResult<Self::MTNode>;
}

pub trait MerkleTree: MerkleTreeBase {
    fn new() -> Self;
    fn to_f3g(&self, p_be: &mut Vec<F3G>);
    fn merkelize(&mut self, buff: Vec<FGL>, width: usize, height: usize) -> EigenResult<()>;
    fn get_element(&self, idx: usize, sub_idx: usize) -> FGL;
    fn get_group_proof(&self, idx: usize) -> EigenResult<(Vec<FGL>, Vec<Vec<Self::BaseField>>)>;
    fn verify_group_proof(
        &self,
        root: &Self::MTNode,
        mp: &Vec<Vec<Self::BaseField>>,
        idx: usize,
        group_elements: &Vec<FGL>,
    ) -> EigenResult<bool>;
    fn root(&self) -> Self::MTNode;
    fn eq_root(&self, r1: &Self::MTNode, r2: &Self::MTNode) -> bool;
    fn element_size(&self) -> usize;
    
}

pub trait Transcript {
    fn new() -> Self;
    fn get_field(&mut self) -> F3G;
    fn get_fields1(&mut self) -> EigenResult<FGL>;
    fn put(&mut self, es: &[Vec<FGL>]) -> EigenResult<()>;
    fn get_permutations(&mut self, n: usize, nbits: usize) -> EigenResult<Vec<usize>>;
}

pub trait PoseidonTrait {
    type BaseField;
    fn new() -> Self;
    fn poseidon_hash(&self, inp: &Vec<Self::BaseField>, init_state: &[Self::BaseField], out: usize) -> Result<Vec<Self::BaseField>, String>;
}
