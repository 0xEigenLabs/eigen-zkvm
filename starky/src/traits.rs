use crate::errors::Result;
use crate::f3g::F3G;
use ff::PrimeField;
use plonky::field_gl::Fr as FGL;

pub trait MTNodeType {
    fn as_elements(&self) -> &[FGL];
    fn new(value: &[FGL]) -> Self;
    fn from_scalar<T: PrimeField>(e: &T) -> Self;
    fn as_bn128(self) -> crate::field_bn128::Fr;
}

pub trait MerkleTree {
    type MTNode: Copy + std::fmt::Display + Clone + Default + MTNodeType + core::fmt::Debug;
    type BaseField: Clone
        + Default
        + core::fmt::Debug
        + Into<crate::serializer::Input<Self::MTNode>>;
    fn new() -> Self;
    fn to_f3g(&self, p_be: &mut Vec<F3G>);
    fn merkelize(&mut self, buff: Vec<FGL>, width: usize, height: usize) -> Result<()>;
    fn get_element(&self, idx: usize, sub_idx: usize) -> FGL;
    fn get_group_proof(&self, idx: usize) -> Result<(Vec<FGL>, Vec<Vec<Self::BaseField>>)>;
    fn verify_group_proof(
        &self,
        root: &Self::MTNode,
        mp: &Vec<Vec<Self::BaseField>>,
        idx: usize,
        group_elements: &Vec<FGL>,
    ) -> Result<bool>;
    fn root(&self) -> Self::MTNode;
    fn eq_root(&self, r1: &Self::MTNode, r2: &Self::MTNode) -> bool;
    fn element_size(&self) -> usize;
}

pub trait Transcript {
    fn new() -> Self;
    fn get_field(&mut self) -> F3G;
    fn get_fields1(&mut self) -> Result<FGL>;
    fn put(&mut self, es: &[Vec<FGL>]) -> Result<()>;
    fn get_permutations(&mut self, n: usize, nbits: usize) -> Result<Vec<usize>>;
}
