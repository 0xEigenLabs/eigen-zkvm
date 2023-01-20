use crate::digest::ElementDigest;
use crate::errors::Result;
use crate::f3g::F3G;
use winter_math::fields::f64::BaseElement;

pub trait MerkleTree {
    type BaseField: Clone + std::default::Default + Into<crate::serializer::Input>;
    fn new() -> Self;
    fn to_f3g(&self, p_be: &mut Vec<F3G>);
    fn merkelize(&mut self, buff: Vec<BaseElement>, width: usize, height: usize) -> Result<()>;
    fn get_element(&self, idx: usize, sub_idx: usize) -> BaseElement;
    fn get_group_proof(&self, idx: usize) -> Result<(Vec<BaseElement>, Vec<Vec<Self::BaseField>>)>;
    fn verify_group_proof(
        &self,
        root: &ElementDigest,
        mp: &Vec<Vec<Self::BaseField>>,
        idx: usize,
        group_elements: &Vec<BaseElement>,
    ) -> Result<bool>;
    fn root(&self) -> ElementDigest;
    fn eq_root(&self, r1: &ElementDigest, r2: &ElementDigest) -> bool;
    fn element_size(&self) -> usize;
}

pub trait Transcript {
    fn new() -> Self;
    fn get_field(&mut self) -> F3G;
    fn get_fields1(&mut self) -> Result<BaseElement>;
    fn put(&mut self, es: &[ElementDigest]) -> Result<()>;
    fn get_permutations(&mut self, n: usize, nbits: usize) -> Result<Vec<usize>>;
}
