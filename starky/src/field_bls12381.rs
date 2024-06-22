#![allow(unused_imports, clippy::too_many_arguments)]
use core::ops::{Add, Div, Mul, Neg, Sub};
use ff::*;

use crate::helper;
use serde::de::{Error, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Serialize, Deserialize, PrimeField)]
#[PrimeFieldModulus = "52435875175126190479447740508185965837690552500527637822603658699938581184513"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
pub struct Fr([u64; 4]);

// impl Serialize for Fr {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_str(&helper::fr_to_biguint(self).to_string())
//     }
// }

// impl<'de> Deserialize<'de> for Fr {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct EntriesVisitor;

//         impl<'de> Visitor<'de> for EntriesVisitor {
//             type Value = Fr;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("struct Bls12381's Fr")
//             }

//             fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
//             where
//                 E: Error,
//             {
//                 Ok(Self::Value::from_str(v).unwrap())
//             }
//         }
//         deserializer.deserialize_any(EntriesVisitor)
//     }
// }

#[cfg(test)]
mod tests {
    use crate::field_bls12381::*;
    use ff::*;
    use ff::{Field, PrimeField};
    use num_bigint::BigInt;
    use rand::rngs::OsRng;
    use std::ops::{Add, Mul, Neg, Sub};

    #[test]
    fn test_ff_bls12381_add() {
        let mut f1: Fr = Fr([
            14416697486971305484u64,
            12900705632832856697u64,
            7610670501874103154u64,
            37415040251072941u64,
        ]);
        let f2: Fr = Fr([
            8180603016049782712u64,
            18970485755295250u64,
            11504259147723040819u64,
            766371471703065751u64,
        ]);
        let f3: Fr = Fr([
            4150556429311536580u64,
            12919676118588151948u64,
            668185575887592357u64,
            803786511954138693u64,
        ]);
        f1 = f1 + f2;
        assert_eq!(f1, f3);
    }

    #[test]
    fn test_ff_bls12381_mul() {
        let v = Fr::random(OsRng);
        let lhs = v * v * v;
        let mut rhs = v.square();
        rhs = rhs * v;
        assert_eq!(lhs, rhs);
    }
}
