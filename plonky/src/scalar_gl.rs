#![allow(unused_imports)]
use crate::ff::*;
use crate::bellman_ce::ScalarEngine;

#[derive(PrimeField)]
#[PrimeFieldModulus = "18446744069414584321"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(pub FrRepr);


#[derive(Clone, Copy, Debug)]
pub struct GL;

impl ScalarEngine for GL {
    type Fr = Fr;
}
