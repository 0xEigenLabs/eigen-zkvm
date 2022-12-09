use crate::ElementDigest;
use ff::{Field, PrimeField, PrimeFieldRepr};
use rayon::prelude::*;
use winter_crypto::Hasher;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(pub FrRepr);
