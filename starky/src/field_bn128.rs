#![allow(unused_imports, clippy::too_many_arguments)]

use crate::f3g::F3G;
use crate::helper;
use ff::*;
use serde::de::{Error, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(pub FrRepr);

impl Serialize for Fr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&helper::fr_to_biguint(self).to_string())
    }
}

impl<'de> Deserialize<'de> for Fr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntriesVisitor;

        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = Fr;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Bn128's Fr")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Self::Value::from_str(v).unwrap())
            }
        }
        deserializer.deserialize_any(EntriesVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::field_bn128::*;
    use ff::*;

    #[test]
    fn test_ff() {
        let a = Fr::from_repr(FrRepr::from(2)).unwrap();
        assert_eq!(
            "0000000000000000000000000000000000000000000000000000000000000002",
            to_hex(&a)
        );

        let b: Fr = Fr::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495619",
        )
        .unwrap();
        assert_eq!(
            "0000000000000000000000000000000000000000000000000000000000000002",
            to_hex(&b)
        );
        assert_eq!(&a, &b);
    }

    #[test]
    fn test_bn128_fr_serde_and_deserde() {
        let data = Fr::one();
        let serialized = serde_json::to_string(&data).unwrap();
        println!("Serialized: {}", serialized);

        let expect: Fr = serde_json::from_str(&serialized).unwrap();

        assert_eq!(data, expect);
    }
}
