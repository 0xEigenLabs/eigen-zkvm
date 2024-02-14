// input json of plonk
#![allow(non_snake_case)]
use crate::f3g::F3G;
use crate::f5g::F5G;
use crate::field_bls12381::Fr as Fr_bls12381;
use crate::field_bn128::Fr;
use crate::helper;
use crate::stark_gen::StarkProof;
use crate::traits::FieldExtension;
use crate::traits::{MTNodeType, MerkleTree};
use fields::field_gl::Fr as FGL;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;
use std::marker::PhantomData;

impl Serialize for F3G {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let elems = self.as_elements();
        if self.dim == 1 {
            serializer.serialize_str(&elems[0].as_int().to_string())
        } else if self.dim == 3 {
            let mut seq = serializer.serialize_seq(Some(elems.len()))?;
            for v in elems.iter() {
                seq.serialize_element(&v.as_int().to_string())?;
            }
            seq.end()
        } else {
            panic!("Invalid dim {}", self);
        }
    }
}

impl<'de> Deserialize<'de> for F3G {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntriesVisitor;

        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = F3G;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct F3G")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some(entry) = seq.next_element::<String>()? {
                    let entry: u64 = entry.parse().unwrap();
                    entries.push(FGL::from(entry));
                }
                Ok(F3G::from_vec(entries))
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let ien: u64 = s.parse().unwrap();
                Ok(F3G::from(ien))
            }
        }
        deserializer.deserialize_any(EntriesVisitor)
    }
}

impl Serialize for F5G {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let elems = self.as_elements();
        if self.dim == 1 {
            serializer.serialize_str(&elems[0].as_int().to_string())
        } else if self.dim == 5 {
            let mut seq = serializer.serialize_seq(Some(elems.len()))?;
            for v in elems.iter() {
                seq.serialize_element(&v.as_int().to_string())?;
            }
            seq.end()
        } else {
            panic!("Invalid dim {}", self);
        }
    }
}

impl<'de> Deserialize<'de> for F5G {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntriesVisitor;

        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = F5G;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct F5G")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some(entry) = seq.next_element::<String>()? {
                    let entry: u64 = entry.parse().unwrap();
                    entries.push(FGL::from(entry));
                }
                Ok(F5G::from_vec(entries))
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let ien: u64 = s.parse().unwrap();
                Ok(F5G::from(ien))
            }
        }
        deserializer.deserialize_any(EntriesVisitor)
    }
}

pub struct NodeWrapper<T: MTNodeType>(T, String);

impl<T: MTNodeType> NodeWrapper<T> {
    pub fn new(e: T, hashtype: String) -> Self {
        NodeWrapper(e, hashtype)
    }
    pub fn is_dim_1(&self) -> bool {
        let e = self.0.as_elements();
        e[1] == e[2] && e[1] == e[3] && e[1] == FGL::ZERO
    }
}

impl<T: MTNodeType + Clone> Serialize for NodeWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.1.as_str() {
            "BN128" => {
                let r: Fr = Fr(self.0.clone().as_scalar::<Fr>());
                serializer.serialize_str(&helper::fr_to_biguint(&r).to_string())
            }
            "BLS12381" => {
                let r: Fr_bls12381 = Fr_bls12381(self.0.clone().as_scalar::<Fr_bls12381>());
                serializer.serialize_str(&helper::fr_bls12381_to_biguint(&r).to_string())
            }
            "GL" => {
                let e = self.0.as_elements();
                if self.is_dim_1() {
                    serializer.serialize_str(&e[0].as_int().to_string())
                } else {
                    let mut seq = serializer.serialize_seq(Some(4))?;
                    for v in e.iter() {
                        seq.serialize_element(&v.as_int().to_string())?;
                    }
                    seq.end()
                }
            }
            _ => panic!("Invalid hashtype {}", self.1),
        }
    }
}

impl<'de, T: MTNodeType> Deserialize<'de> for NodeWrapper<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntriesVisitor<MT: MTNodeType>(PhantomData<MT>);

        impl<'de, MT: MTNodeType> Visitor<'de> for EntriesVisitor<MT> {
            type Value = NodeWrapper<MT>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct NodeWrapper")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entries = Vec::new();
                while let Some(entry) = seq.next_element::<String>()? {
                    let entry: u64 = entry.parse().unwrap();
                    entries.push(FGL::from(entry));
                }
                Ok(NodeWrapper(MT::new(&entries), "GL".to_string()))
            }

            // it could be one-dim GL, BN128, or BLS12381
            /*
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let t = MTNodeType::from_scalar;
                Ok(
                    NodeWrapper::from()
                )
            }
            */
        }
        deserializer.deserialize_any(EntriesVisitor::<T>(Default::default()))
    }
}

impl<T: MTNodeType> From<Fr> for NodeWrapper<T> {
    fn from(val: Fr) -> Self {
        let e = T::from_scalar(&val);
        Self(e, "BN128".to_string())
    }
}

impl<T: MTNodeType> From<Fr_bls12381> for NodeWrapper<T> {
    fn from(val: Fr_bls12381) -> Self {
        let e = T::from_scalar(&val);
        Self(e, "BLS12381".to_string())
    }
}

impl<T: MTNodeType> From<FGL> for NodeWrapper<T> {
    fn from(val: FGL) -> Self {
        Self(
            T::new(&[val, FGL::ZERO, FGL::ZERO, FGL::ZERO]),
            "GL".to_string(),
        )
    }
}

impl<M: MerkleTree> Serialize for StarkProof<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // root, evals, friProof * 3, s0_val{1,2,3,4,C},  s0_siblings{1,2,3,4,C}, finalPol
        let len = 16 + (self.fri_proof.queries.len() - 1) * 3;
        let mut map = serializer.serialize_map(Some(len))?;

        let hashtype = &self.stark_struct.verificationHashType;
        match &self.rootC {
            Some(value) => {
                map.serialize_entry(
                    "rootC",
                    &NodeWrapper::<M::MTNode>::new(*value, hashtype.clone()),
                )?;
            }
            None => {}
        }
        map.serialize_entry(
            "root1",
            &NodeWrapper::<M::MTNode>::new(self.root1, hashtype.clone()),
        )?;
        map.serialize_entry(
            "root2",
            &NodeWrapper::<M::MTNode>::new(self.root2, hashtype.clone()),
        )?;
        map.serialize_entry(
            "root3",
            &NodeWrapper::<M::MTNode>::new(self.root3, hashtype.clone()),
        )?;
        map.serialize_entry(
            "root4",
            &NodeWrapper::<M::MTNode>::new(self.root4, hashtype.clone()),
        )?;
        map.serialize_entry("evals", &self.evals)?;

        for i in 1..(self.fri_proof.queries.len()) {
            map.serialize_entry(
                &format!("s{}_root", i),
                &NodeWrapper::new(self.fri_proof.queries[i].root, hashtype.clone()),
            )?;
            let mut vals: Vec<Vec<F3G>> = vec![];
            let mut sibs: Vec<Vec<Vec<NodeWrapper<M::MTNode>>>> = vec![];
            for q in 0..self.fri_proof.queries[0].pol_queries.len() {
                let qe = &self.fri_proof.queries[i].pol_queries[q];
                vals.push(qe[0].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
                sibs.push(
                    qe[0]
                        .1
                        .iter()
                        .map(|e| {
                            e.iter()
                                .map(|ee| {
                                    let mut res: NodeWrapper<M::MTNode> = ee.clone().into();
                                    res.1 = hashtype.clone();
                                    res
                                })
                                .collect::<Vec<NodeWrapper<M::MTNode>>>()
                        })
                        .collect::<Vec<Vec<NodeWrapper<M::MTNode>>>>(),
                );
            }
            map.serialize_entry(&format!("s{}_vals", i), &vals)?;
            map.serialize_entry(&format!("s{}_siblings", i), &sibs)?;
        }

        let mut s0_vals1: Vec<Vec<F3G>> = vec![];
        let mut s0_vals2: Vec<Vec<F3G>> = vec![];
        let mut s0_vals3: Vec<Vec<F3G>> = vec![];
        let mut s0_vals4: Vec<Vec<F3G>> = vec![];
        let mut s0_valsC: Vec<Vec<F3G>> = vec![];
        let mut s0_siblings1: Vec<Vec<Vec<NodeWrapper<M::MTNode>>>> = vec![];
        let mut s0_siblings2: Vec<Vec<Vec<NodeWrapper<M::MTNode>>>> = vec![];
        let mut s0_siblings3: Vec<Vec<Vec<NodeWrapper<M::MTNode>>>> = vec![];
        let mut s0_siblings4: Vec<Vec<Vec<NodeWrapper<M::MTNode>>>> = vec![];
        let mut s0_siblingsC: Vec<Vec<Vec<NodeWrapper<M::MTNode>>>> = vec![];

        for i in 0..self.fri_proof.queries[0].pol_queries.len() {
            //(leaf, path) represents each query
            let qe = &self.fri_proof.queries[0].pol_queries[i];
            s0_vals1.push(qe[0].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
            s0_siblings1.push(
                qe[0]
                    .1
                    .iter()
                    .map(|e| {
                        e.iter()
                            .map(|ee| {
                                let mut res: NodeWrapper<M::MTNode> = ee.clone().into();
                                res.1 = hashtype.clone();
                                res
                            })
                            .collect::<Vec<NodeWrapper<M::MTNode>>>()
                    })
                    .collect::<Vec<Vec<NodeWrapper<M::MTNode>>>>(),
            );

            if !qe[1].0.is_empty() {
                s0_vals2.push(qe[1].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
                s0_siblings2.push(
                    qe[1]
                        .1
                        .iter()
                        .map(|e| {
                            e.iter()
                                .map(|ee| {
                                    let mut res: NodeWrapper<M::MTNode> = ee.clone().into();
                                    res.1 = hashtype.clone();
                                    res
                                })
                                .collect::<Vec<NodeWrapper<M::MTNode>>>()
                        })
                        .collect::<Vec<Vec<NodeWrapper<M::MTNode>>>>(),
                );
            }

            if !qe[2].0.is_empty() {
                s0_vals3.push(qe[2].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
                s0_siblings3.push(
                    qe[2]
                        .1
                        .iter()
                        .map(|e| {
                            e.iter()
                                .map(|ee| {
                                    let mut res: NodeWrapper<M::MTNode> = ee.clone().into();
                                    res.1 = hashtype.clone();
                                    res
                                })
                                .collect::<Vec<NodeWrapper<M::MTNode>>>()
                        })
                        .collect::<Vec<Vec<NodeWrapper<M::MTNode>>>>(),
                );
            }

            let qe = &self.fri_proof.queries[0].pol_queries[i];
            if !qe[3].0.is_empty() {
                s0_vals4.push(qe[3].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
                s0_siblings4.push(
                    qe[3]
                        .1
                        .iter()
                        .map(|e| {
                            e.iter()
                                .map(|ee| {
                                    let mut res: NodeWrapper<M::MTNode> = ee.clone().into();
                                    res.1 = hashtype.clone();
                                    res
                                })
                                .collect::<Vec<NodeWrapper<M::MTNode>>>()
                        })
                        .collect::<Vec<Vec<NodeWrapper<M::MTNode>>>>(),
                );
            }

            if !qe[4].0.is_empty() {
                s0_valsC.push(qe[4].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
                s0_siblingsC.push(
                    qe[4]
                        .1
                        .iter()
                        .map(|e| {
                            e.iter()
                                .map(|ee| {
                                    let mut res: NodeWrapper<M::MTNode> = ee.clone().into();
                                    res.1 = hashtype.clone();
                                    res
                                })
                                .collect::<Vec<NodeWrapper<M::MTNode>>>()
                        })
                        .collect::<Vec<Vec<NodeWrapper<M::MTNode>>>>(),
                );
            }
        }

        map.serialize_entry("s0_vals1", &s0_vals1)?;
        if !s0_vals2.is_empty() {
            map.serialize_entry("s0_vals2", &s0_vals2)?;
        }
        if !s0_vals3.is_empty() {
            map.serialize_entry("s0_vals3", &s0_vals3)?;
        }
        map.serialize_entry("s0_vals4", &s0_vals4)?;
        map.serialize_entry("s0_valsC", &s0_valsC)?;
        map.serialize_entry("s0_siblings1", &s0_siblings1)?;
        if !s0_siblings2.is_empty() {
            map.serialize_entry("s0_siblings2", &s0_siblings2)?;
        }
        if !s0_siblings3.is_empty() {
            map.serialize_entry("s0_siblings3", &s0_siblings3)?;
        }
        map.serialize_entry("s0_siblings4", &s0_siblings4)?;
        map.serialize_entry("s0_siblingsC", &s0_siblingsC)?;
        map.serialize_entry("finalPol", &self.fri_proof.last)?;
        map.serialize_entry("publics", &self.publics)?;
        if hashtype.as_str() == "BN128" || hashtype.as_str() == "BLS12381" {
            map.serialize_entry("proverAddr", &self.prover_addr)?;
        }
        map.end()
    }
}

/*
impl<'de, M: MerkleTree> Deserialize<'de> for StarkProof<M> {
fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
where D: Deserializer<'de> {


}
}
*/

#[cfg(test)]
mod tests {
    use crate::f3g::F3G;
    use crate::f5g::F5G;
    use crate::traits::FieldExtension;
    use fields::field_gl::Fr as FGL;
    use rand::Rand;

    #[test]
    fn test_serialize_f3g() {
        let input = F3G::from(123);
        let ser_input = serde_json::to_string(&input).unwrap();
        let de_input = serde_json::from_str(&ser_input).unwrap();
        assert_eq!(input, de_input);

        let mut rng = rand::thread_rng();
        let input = F3G::from_vec(
            [
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
            ]
            .to_vec(),
        );
        let ser_input = serde_json::to_string(&input).unwrap();
        let de_input = serde_json::from_str(&ser_input).unwrap();
        assert_eq!(input, de_input);
    }

    #[test]
    fn test_serialize_f5g() {
        let input = F5G::from(123);
        let ser_input = serde_json::to_string(&input).unwrap();
        let de_input = serde_json::from_str(&ser_input).unwrap();
        assert_eq!(input, de_input);

        let mut rng = rand::thread_rng();
        let input = F5G::from_vec(
            [
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
                FGL::rand(&mut rng),
            ]
            .to_vec(),
        );
        let ser_input = serde_json::to_string(&input).unwrap();
        let de_input = serde_json::from_str(&ser_input).unwrap();
        assert_eq!(input, de_input);
    }
    fn test_stark_proof_ser_der() {}
}
