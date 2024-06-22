// use ::rand::Rand;
use anyhow::Result;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use ff::Field;
use ff::PrimeField;
use fields::field_gl::Goldilocks as FGL;
use serde::{de::DeserializeOwned, ser::Serialize};
use std::fmt::{Debug, Display};
use std::hash::Hash;

pub trait MTNodeType
where
    Self: Sized + PartialEq + Debug,
{
    // BaseField is for type derivation when serializing proof
    type BaseField: PrimeField + Default;
    fn as_elements(&self) -> &[FGL];
    fn new(value: &[FGL]) -> Self;
    fn from_scalar<T: PrimeField>(e: &T) -> Self;
    fn as_scalar<T: PrimeField>(&self) -> T::Repr;
}

#[allow(clippy::type_complexity)]
pub trait MerkleTree
where
    Self: Sized,
{
    type MTNode: Copy
        + Display
        + Clone
        + Default
        + MTNodeType
        + Debug
        + Serialize
        + DeserializeOwned;
    type ExtendField: FieldExtension;
    type BaseField: Clone + Default + Debug + PartialEq + Serialize + DeserializeOwned;
    fn new() -> Self;
    fn to_extend(&self, p_be: &mut Vec<Self::ExtendField>);
    fn to_basefield(node: &Self::MTNode) -> Vec<Self::BaseField>;
    fn from_basefield(node: &Self::BaseField) -> Self::MTNode;
    fn merkelize(&mut self, buff: Vec<FGL>, width: usize, height: usize) -> Result<()>;
    fn get_element(&self, idx: usize, sub_idx: usize) -> FGL;
    fn get_group_proof(&self, idx: usize) -> Result<(Vec<FGL>, Vec<Vec<Self::BaseField>>)>;
    fn verify_group_proof(
        &self,
        root: &Self::MTNode,
        mp: &[Vec<Self::BaseField>],
        idx: usize,
        group_elements: &[FGL],
    ) -> Result<bool>;
    fn root(&self) -> Self::MTNode;
    fn eq_root(&self, r1: &Self::MTNode, r2: &Self::MTNode) -> bool;
    fn element_size(&self) -> usize;
}

pub trait Transcript {
    fn new() -> Self;
    fn get_field<F: FieldExtension>(&mut self) -> F;
    fn get_fields1(&mut self) -> Result<FGL>;
    fn put(&mut self, es: &[Vec<FGL>]) -> Result<()>;
    fn get_permutations(&mut self, n: usize, nbits: usize) -> Result<Vec<usize>>;
}

pub trait FieldExtension:
    From<FGL>
    + From<u64>
    + From<i32>
    + From<usize>
    + Debug
    + Hash
    + Copy
    + Clone
    + PartialEq
    + Eq
    + Default
    + Add<Output = Self>
    + AddAssign
    + Mul<Output = Self>
    + MulAssign
    + Neg<Output = Self>
    + Sub<Output = Self>
    + SubAssign
    + Display
    + Send
    + Sync
    + Field
    + Serialize
    + DeserializeOwned
{
    const ELEMENT_BYTES: usize;
    const IS_CANONICAL: bool = false;
    // const ZERO: Self;
    // const ONE: Self;

    const ZEROS: Self;
    const ONES: Self;
    const NEW_SIZE: u64 = 0;
    fn dim(&self) -> usize;
    fn from_vec(values: Vec<FGL>) -> Self;
    fn to_be(&self) -> FGL;
    fn as_elements(&self) -> Vec<FGL>;
    fn mul_scalar(&self, b: usize) -> Self;
    fn _eq(&self, rhs: &Self) -> bool;
    fn gt(&self, rhs: &Self) -> bool;
    fn geq(&self, rhs: &Self) -> bool;
    fn lt(&self, rhs: &Self) -> bool;
    fn leq(&self, rhs: &Self) -> bool;
    fn exp(&self, e_: usize) -> Self;
    fn inv(&self) -> Self;
    fn as_int(&self) -> u64;
    fn elements_as_bytes(elements: &[Self]) -> &[u8];
    fn as_bytes(&self) -> &[u8];
    // TODO: Add generate rand fields vector for test/dev.
    // fn rand_
    // (&self) -> &[u8];
}

// This is only for proof serializer
#[inline]
pub(crate) fn mt_node_to_basefield<M: MerkleTree>(
    e2d: &[Vec<M::MTNode>],
) -> Vec<Vec<M::BaseField>> {
    let mut res: Vec<Vec<M::BaseField>> = vec![vec![]; e2d.len()];
    for i in 0..e2d.len() {
        for j in 0..e2d[i].len() {
            let mut t = M::to_basefield(&e2d[i][j]);
            res[i].append(&mut t);
        }
    }
    res
}
