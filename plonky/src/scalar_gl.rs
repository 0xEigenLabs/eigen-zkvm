#![allow(unused_imports)]
use crate::bellman_ce::ScalarEngine;
use crate::ff::*;

#[derive(PrimeField)]
#[PrimeFieldModulus = "18446744069414584321"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(pub FrRepr);

#[derive(Clone, Copy, Debug)]
pub struct GL;

impl ScalarEngine for GL {
    type Fr = Fr;
}
