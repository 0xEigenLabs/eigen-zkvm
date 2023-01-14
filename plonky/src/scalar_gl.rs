#![allow(unused_imports)]
use ff::*;

use crate::bellman_ce::pairing::{Engine, CurveProjective};
use std::default::Default;

#[derive(PrimeField)]
#[PrimeFieldModulus = "18446744069414584321"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(pub FrRepr);


#[derive(Clone, Copy, Debug)]
pub struct GL;

impl ScalarEngine for GL {
    type Fr = Fr;
}
