// input json of plonk
#![allow(non_snake_case)]
use crate::f3g::F3G;
use crate::f5g::F5G;
use crate::field_bls12381::Fr as Fr_BLS12381;
use crate::field_bn128::Fr;
use crate::fri::FRIProof;
use crate::fri::Query;
use crate::helper;
use crate::stark_gen::StarkProof;
use crate::traits::FieldExtension;
use crate::traits::{MTNodeType, MerkleTree};
use ff::{to_hex, PrimeField};
use fields::field_gl::Fr as FGL;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::any::TypeId;
use std::collections::HashMap;
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

// Is it making sense to wrap?
#[derive(Clone)]
pub struct NodeWrapper<T: MTNodeType>(T);

impl<T: MTNodeType> NodeWrapper<T> {
    pub fn new(e: T) -> Self {
        NodeWrapper(e)
    }
    pub fn is_dim_1(&self) -> bool {
        let e = self.0.as_elements();
        e[1] == e[2] && e[1] == e[3] && e[1] == FGL::ZERO
    }
}

impl<T: MTNodeType + fmt::Debug + Clone> Serialize for NodeWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let source = TypeId::of::<T::FieldType>();
        if source == TypeId::of::<Fr>() {
            let r: Fr = Fr(self.0.clone().as_scalar::<Fr>());
            return serializer.serialize_str(&helper::fr_to_biguint(&r).to_string());
        }
        if source == TypeId::of::<Fr_BLS12381>() {
            let r: Fr_BLS12381 = Fr_BLS12381(self.0.clone().as_scalar::<Fr_BLS12381>());
            return serializer.serialize_str(&helper::fr_bls12381_to_biguint(&r).to_string());
        }
        if source == TypeId::of::<FGL>() {
            let e = self.0.as_elements();
            if self.is_dim_1() {
                return serializer.serialize_str(&e[0].as_int().to_string());
            } else {
                let mut seq = serializer.serialize_seq(Some(4))?;
                for v in e.iter() {
                    seq.serialize_element(&v.as_int().to_string())?;
                }
                return seq.end();
            }
        }
        panic!("Invalid element for seralizing, {:?}", self.0)
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
                Ok(NodeWrapper(MT::new(&entries)))
            }

            // it could be one-dim GL, BN128, or BLS12381
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let source = TypeId::of::<MT::FieldType>();
                if source == TypeId::of::<FGL>() {
                    // one-dim GL elements
                    let value = FGL::from_str(s).unwrap();
                    let one_fgl: NodeWrapper<MT> = NodeWrapper::from(value);
                    Ok(one_fgl)
                } else {
                    // BN128 or BLS12381
                    let t = <MT as MTNodeType>::FieldType::from_str(s).unwrap();
                    Ok(NodeWrapper(MT::from_scalar(&t)))
                }
            }
        }
        deserializer.deserialize_any(EntriesVisitor::<T>(Default::default()))
    }
}

impl<T: MTNodeType> From<Fr> for NodeWrapper<T> {
    fn from(val: Fr) -> Self {
        let e = T::from_scalar(&val);
        Self(e)
    }
}

impl<T: MTNodeType> From<Fr_BLS12381> for NodeWrapper<T> {
    fn from(val: Fr_BLS12381) -> Self {
        let e = T::from_scalar(&val);
        Self(e)
    }
}

impl<T: MTNodeType> From<FGL> for NodeWrapper<T> {
    fn from(val: FGL) -> Self {
        Self(T::new(&[val, FGL::ZERO, FGL::ZERO, FGL::ZERO]))
    }
}

// convert FieldType to MTNode: convert FieldType to specific PrimeField, like FGL, or BN128.
// then we can use NodeWrapper::from to get the target type.
fn to_<M: MerkleTree>(e: &<M::MTNode as MTNodeType>::FieldType) -> NodeWrapper<M::MTNode> {
    let source = TypeId::of::<<M::MTNode as MTNodeType>::FieldType>();
    if source == TypeId::of::<FGL>() {
        // one-dim GL elements, it's hacky.
        let repr = to_hex(e);
        let repr_10 = u64::from_str_radix(&repr, 16).unwrap();
        let val: FGL = FGL::from(repr_10);
        NodeWrapper::from(val)
    } else {
        // BN128 or BLS12381
        let e = <M::MTNode as MTNodeType>::from_scalar(e);
        NodeWrapper(e)
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

        match &self.rootC {
            Some(value) => {
                map.serialize_entry("rootC", &NodeWrapper::<M::MTNode>::new(*value))?;
            }
            None => {}
        }
        map.serialize_entry("root1", &NodeWrapper::<M::MTNode>::new(self.root1))?;
        map.serialize_entry("root2", &NodeWrapper::<M::MTNode>::new(self.root2))?;
        map.serialize_entry("root3", &NodeWrapper::<M::MTNode>::new(self.root3))?;
        map.serialize_entry("root4", &NodeWrapper::<M::MTNode>::new(self.root4))?;
        map.serialize_entry("evals", &self.evals)?;

        for i in 1..(self.fri_proof.queries.len()) {
            map.serialize_entry(
                &format!("s{}_root", i),
                &NodeWrapper::new(self.fri_proof.queries[i].root),
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
                                .map(|ee| to_::<M>(ee))
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
                            .map(|ee| to_::<M>(ee))
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
                                .map(|ee| to_::<M>(ee))
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
                                .map(|ee| to_::<M>(ee))
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
                                .map(|ee| to_::<M>(ee))
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
                                .map(|ee| to_::<M>(ee))
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

        let source = TypeId::of::<<M::MTNode as MTNodeType>::FieldType>();
        if source != TypeId::of::<FGL>() {
            //hashtype.as_str() == "BN128" || hashtype.as_str() == "BLS12381"
            map.serialize_entry("proverAddr", &self.prover_addr)?;
        }
        map.end()
    }
}

impl<'de, T: MerkleTree + Default> Deserialize<'de> for StarkProof<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntriesVisitor<MT: MerkleTree>(PhantomData<MT>);

        impl<'de, MT: MerkleTree + Default> Visitor<'de> for EntriesVisitor<MT> {
            type Value = StarkProof<MT>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct StarkProof")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut map: HashMap<String, serde_json::Value> =
                    HashMap::with_capacity(access.size_hint().unwrap_or(0));
                while let Some((key, value)) = access.next_entry()? {
                    map.insert(key, value);
                }
                let mut sp: StarkProof<MT> = Default::default();
                let root: NodeWrapper<MT::MTNode> =
                    serde_json::from_value(map.get(&"root1".to_string()).unwrap().clone()).unwrap();
                sp.root1 = root.0;

                let root: NodeWrapper<MT::MTNode> =
                    serde_json::from_value(map.get(&"root2".to_string()).unwrap().clone()).unwrap();
                sp.root2 = root.0;

                let root: NodeWrapper<MT::MTNode> =
                    serde_json::from_value(map.get(&"root3".to_string()).unwrap().clone()).unwrap();
                sp.root3 = root.0;

                let root: NodeWrapper<MT::MTNode> =
                    serde_json::from_value(map.get(&"root4".to_string()).unwrap().clone()).unwrap();
                sp.root4 = root.0;

                let root = map.get(&"rootC".to_string());
                if root.is_some() {
                    let root: NodeWrapper<MT::MTNode> =
                        serde_json::from_value(root.unwrap().clone()).unwrap();
                    sp.rootC = Some(root.0);
                }

                let prover_addr = map.get(&"proverAddr".to_string());
                if prover_addr.is_some() {
                    sp.prover_addr = serde_json::from_value(prover_addr.unwrap().clone()).unwrap();
                }
                sp.evals =
                    serde_json::from_value(map.get(&"evals".to_string()).unwrap().clone()).unwrap();

                sp.publics =
                    serde_json::from_value(map.get(&"publics".to_string()).unwrap().clone())
                        .unwrap();

                let mut fri_proof: FRIProof<MT::ExtendField, MT> = FRIProof::default();

                // search all s{i}_root keys, to avoid regex matching, we assume the max query is
                // less than 32
                let num_query: usize = (1..32)
                    .map(|i| {
                        let key = map.get(&format!("s{}_root", i));
                        if key.is_some() {
                            1
                        } else {
                            0
                        }
                    })
                    .sum();

                fri_proof.queries = vec![Query::default(); num_query + 1];
                // handle query 1 to num_query
                for i in 1..=num_query {
                    let key = map.get(&format!("s{}_root", i));
                    let root: NodeWrapper<MT::MTNode> =
                        serde_json::from_value(key.unwrap().clone()).unwrap();
                    fri_proof.queries[i].root = root.0;

                    let key = map.get(&format!("s{}_vals", i));
                    let val: Vec<Vec<F3G>> = serde_json::from_value(key.unwrap().clone()).unwrap();
                    let vals: Vec<Vec<FGL>> = val
                        .iter()
                        .map(|e| {
                            let iv: Vec<FGL> = e
                                .iter()
                                .map(|e2| {
                                    let ea = e2.as_elements();
                                    ea[0]
                                })
                                .collect();
                            iv
                        })
                        .collect();

                    let key = map.get(&format!("s{}_siblings", i));
                    let sibs: Vec<Vec<Vec<NodeWrapper<MT::MTNode>>>> =
                        serde_json::from_value(key.unwrap().clone()).unwrap();
                    let sibs: Vec<Vec<Vec<<MT::MTNode as MTNodeType>::FieldType>>> = sibs
                        .iter()
                        .map(|e| {
                            e.iter()
                                .map(|e2| {
                                    e2.iter()
                                        .map(|e3| {
                                            let repr = e3
                                                .0
                                                .as_scalar::<<MT::MTNode as MTNodeType>::FieldType>(
                                                );
                                            <<MT::MTNode as MTNodeType>::FieldType>::from_repr(repr)
                                                .unwrap()
                                        })
                                        .collect()
                                })
                                .collect()
                        })
                        .collect();

                    let pol_query0 = vals.iter().cloned().zip(sibs.iter().cloned()).collect();
                    fri_proof.queries[i].pol_queries.push(pol_query0);
                }

                // handle queries[0]
                for _j in ["1", "2", "3", "4", "C"] {
                    let key = map.get(&"s0_vals{_j}".to_string());
                    if key.is_none() {
                        continue;
                    }
                    let s0_vals: Vec<Vec<F3G>> =
                        serde_json::from_value(key.unwrap().clone()).unwrap();
                    let s0_vals: Vec<Vec<FGL>> = s0_vals
                        .iter()
                        .map(|e| {
                            let iv: Vec<FGL> = e
                                .iter()
                                .map(|e2| {
                                    let ea = e2.as_elements();
                                    ea[0]
                                })
                                .collect();
                            iv
                        })
                        .collect();

                    let key = map.get(&"s0_siblings{_j}".to_string());
                    let s0_siblings: Vec<Vec<Vec<NodeWrapper<MT::MTNode>>>> =
                        serde_json::from_value(key.unwrap().clone()).unwrap();
                    let s0_siblings: Vec<Vec<Vec<<MT::MTNode as MTNodeType>::FieldType>>> =
                        s0_siblings
                            .iter()
                            .map(|e| {
                                e.iter()
                                    .map(|e2| {
                                        e2.iter()
                                            .map(|e3| {
                                                let repr = e3
                                                .0
                                                .as_scalar::<<MT::MTNode as MTNodeType>::FieldType>(
                                                );
                                                <<MT::MTNode as MTNodeType>::FieldType>::from_repr(
                                                    repr,
                                                )
                                                .unwrap()
                                            })
                                            .collect()
                                    })
                                    .collect()
                            })
                            .collect();
                    let pol_query0 = s0_vals
                        .iter()
                        .cloned()
                        .zip(s0_siblings.iter().cloned())
                        .collect();
                    fri_proof.queries[0].pol_queries.push(pol_query0);
                }

                // handle finalPol
                let key = map.get(&"finalPol".to_string());
                fri_proof.last = serde_json::from_value(key.unwrap().clone()).unwrap();
                sp.fri_proof = fri_proof;
                Ok(sp)
            }
        }
        deserializer.deserialize_any(EntriesVisitor::<T>(Default::default()))
    }
}

#[cfg(test)]
mod tests {
    use crate::digest::ElementDigest;
    use crate::f3g::F3G;
    use crate::f5g::F5G;
    use crate::field_bls12381::Fr as Fr_BLS12381;
    use crate::field_bn128::Fr;
    use crate::merklehash_bn128::MerkleTreeBN128;
    use crate::polsarray::PolKind;
    use crate::polsarray::PolsArray;
    use crate::serializer::NodeWrapper;
    use crate::serializer::StarkProof;
    use crate::stark_setup::StarkSetup;
    use crate::traits::FieldExtension;
    use crate::traits::MTNodeType;
    use crate::transcript_bn128::TranscriptBN128;
    use crate::types::load_json;
    use crate::types::StarkStruct;
    use crate::types::PIL;
    use fields::field_gl::Fr as FGL;
    use rand::Rand;
    use std::fs::File;
    use std::io::Write;

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

    #[test]
    fn test_serialize_node_wrapper() {
        env_logger::try_init().unwrap_or_default();
        let mut rng = rand::thread_rng();
        let four_fgl = ElementDigest::<4, FGL>::new(&[
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
        ]);

        let four_fgl = NodeWrapper::<ElementDigest<4, FGL>>::new(four_fgl);
        let four_fgl_ser = serde_json::to_string(&four_fgl).unwrap();
        log::debug!("four_fgl_ser: {:?}", four_fgl_ser);
        let actual_four_fgl: NodeWrapper<ElementDigest<4, FGL>> =
            serde_json::from_str(&four_fgl_ser).unwrap();
        assert_eq!(four_fgl.0, actual_four_fgl.0);

        let one_fgl: NodeWrapper<ElementDigest<4, FGL>> = NodeWrapper::from(FGL::rand(&mut rng));
        let one_fgl_ser = serde_json::to_string(&one_fgl).unwrap();
        log::debug!("one_fgl_ser: {:?}", one_fgl_ser);
        let actual_one_fgl: NodeWrapper<ElementDigest<4, FGL>> =
            serde_json::from_str(&one_fgl_ser).unwrap();
        assert_eq!(one_fgl.0, actual_one_fgl.0);

        let one_fr: NodeWrapper<ElementDigest<4, Fr>> = NodeWrapper::from(Fr::rand(&mut rng));
        let one_fr_ser = serde_json::to_string(&one_fr).unwrap();
        log::debug!("one_fr_ser: {:?}", one_fr_ser);
        let actual_one_fr: NodeWrapper<ElementDigest<4, Fr>> =
            serde_json::from_str(&one_fr_ser).unwrap();
        assert_eq!(one_fr.0, actual_one_fr.0);

        let one_fr: NodeWrapper<ElementDigest<4, Fr_BLS12381>> =
            NodeWrapper::from(Fr_BLS12381::rand(&mut rng));
        let one_fr_ser = serde_json::to_string(&one_fr).unwrap();
        log::debug!("one_fr_bls12381_ser: {:?}", one_fr_ser);
        let actual_one_fr: NodeWrapper<ElementDigest<4, Fr_BLS12381>> =
            serde_json::from_str(&one_fr_ser).unwrap();
        assert_eq!(one_fr.0, actual_one_fr.0);
    }

    #[test]
    fn test_serialize_stark_proof_ser_der() {
        let mut pil = load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/fib.cm").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();

        let setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        let fr_root: Fr = Fr(setup.const_root.as_scalar::<Fr>());
        log::trace!("setup {}", fr_root);

        let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
            cm_pol,
            const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
            "273030697313060285579891744179749754319274977764",
        )
        .unwrap();

        // serde to json
        let serialized = serde_json::to_string(&starkproof).unwrap();
        let mut file = File::create("/tmp/test_stark_proof_serialize.json").unwrap();
        write!(file, "{}", serialized).unwrap();
        // deserialized
        let expected: StarkProof<MerkleTreeBN128> = serde_json::from_str(&serialized).unwrap();
        //log::debug!("stark proof der: {:?}", expected);

        let mut file = File::create("/tmp/test_stark_proof_serialize.expected.json").unwrap();
        let serialized2 = serde_json::to_string(&expected).unwrap();
        write!(file, "{}", serialized2).unwrap();

        // assert
        assert_eq!(expected.root1, starkproof.root1);
        assert_eq!(expected.root2, starkproof.root2);
        assert_eq!(expected.root3, starkproof.root3);
        assert_eq!(expected.root4, starkproof.root4);
        assert_eq!(expected.rootC, starkproof.rootC);
        assert_eq!(expected.publics, starkproof.publics);
        assert_eq!(expected.evals, starkproof.evals);
        assert_eq!(expected.fri_proof, starkproof.fri_proof);
        assert_eq!(expected, starkproof);

        // TODO: test GL and BLS12381
    }
}
