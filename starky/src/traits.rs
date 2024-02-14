use ::rand::Rand;
use anyhow::Result;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use ff::PrimeField;
use fields::field_gl::Fr as FGL;
use fields::field_gl::Fr;
use fields::Field;
use serde::ser::Serialize;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::{Read, Write};

pub trait MTNodeType
where
    Self: Sized,
{
    fn as_elements(&self) -> &[FGL];
    fn new(value: &[FGL]) -> Self;
    fn from_scalar<T: PrimeField>(e: &T) -> Self;
    fn as_scalar<T: PrimeField>(&self) -> T::Repr;
    fn save<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn load<R: Read>(reader: &mut R) -> Result<Self>;
}

#[allow(clippy::type_complexity)]
pub trait MerkleTree
where
    Self: Sized,
{
    type MTNode: Copy + std::fmt::Display + Clone + Default + MTNodeType + core::fmt::Debug;
    type BaseField: Clone
        + Default
        + core::fmt::Debug
        + Into<crate::serializer::NodeWrapper<Self::MTNode>>;
    type ExtendField: FieldExtension;
    fn new() -> Self;
    fn to_extend(&self, p_be: &mut Vec<Self::ExtendField>);
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
    fn save<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn load<R: Read>(reader: &mut R) -> Result<Self>;
}

pub trait Transcript {
    fn new() -> Self;
    fn get_field<F: FieldExtension>(&mut self) -> F;
    fn get_fields1(&mut self) -> Result<FGL>;
    fn put(&mut self, es: &[Vec<FGL>]) -> Result<()>;
    fn get_permutations(&mut self, n: usize, nbits: usize) -> Result<Vec<usize>>;
}

pub trait FieldExtension:
    From<Fr>
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
    + Div<Output = Self>
    + DivAssign
    + Mul<Output = Self>
    + MulAssign
    + Neg<Output = Self>
    + Sub<Output = Self>
    + SubAssign
    + Rand
    + Display
    + Send
    + Sync
    + Field
    + Serialize
{
    const ELEMENT_BYTES: usize;
    const IS_CANONICAL: bool = false;
    const ZERO: Self;
    const ONE: Self;

    const ZEROS: Self;
    const ONES: Self;
    const NEW_SIZE: u64 = 0;
    fn dim(&self) -> usize;
    fn from_vec(values: Vec<Fr>) -> Self;
    fn to_be(&self) -> Fr;
    fn as_elements(&self) -> Vec<Fr>;
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
