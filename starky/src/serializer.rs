// input json of plonk
#![allow(non_snake_case)]
use crate::f3g::F3G;
use crate::f5g::F5G;
use crate::fri::FRIProof;
use crate::fri::Query;
use crate::stark_gen::StarkProof;
use crate::traits::FieldExtension;
use crate::traits::{MTNodeType, MerkleTree};
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

impl<M: MerkleTree> Serialize for StarkProof<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // root, evals, friProof * 3, s0_val{1,2,3,4,C},  s0_siblings{1,2,3,4,C}, finalPol
        let len = 16 + (self.fri_proof.queries.len() - 1) * 3;
        let mut map = serializer.serialize_map(Some(len))?;

        if self.rootC.is_some() {
            map.serialize_entry("rootC", &self.rootC.unwrap())?;
        }

        map.serialize_entry("root1", &self.root1)?;
        map.serialize_entry("root2", &self.root2)?;
        map.serialize_entry("root3", &self.root3)?;
        map.serialize_entry("root4", &self.root4)?;
        map.serialize_entry("evals", &self.evals)?;

        for i in 1..(self.fri_proof.queries.len()) {
            map.serialize_entry(&format!("s{}_root", i), &self.fri_proof.queries[i].root)?;
            let mut vals: Vec<Vec<F3G>> = vec![];
            let mut sibs: Vec<Vec<Vec<M::MTNode>>> = vec![];
            for q in 0..self.fri_proof.queries[0].pol_queries.len() {
                let qe = &self.fri_proof.queries[i].pol_queries[q];
                vals.push(qe[0].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
                sibs.push(
                    qe[0]
                        .1
                        .iter()
                        .map(|e| {
                            e.iter().map(|ee| M::from_basefield(ee)).collect::<Vec<M::MTNode>>()
                        })
                        .collect::<Vec<Vec<M::MTNode>>>(),
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
        let mut s0_siblings1: Vec<Vec<Vec<M::MTNode>>> = vec![];
        let mut s0_siblings2: Vec<Vec<Vec<M::MTNode>>> = vec![];
        let mut s0_siblings3: Vec<Vec<Vec<M::MTNode>>> = vec![];
        let mut s0_siblings4: Vec<Vec<Vec<M::MTNode>>> = vec![];
        let mut s0_siblingsC: Vec<Vec<Vec<M::MTNode>>> = vec![];

        for i in 0..self.fri_proof.queries[0].pol_queries.len() {
            //(leaf, path) represents each query
            let qe = &self.fri_proof.queries[0].pol_queries[i];
            s0_vals1.push(qe[0].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
            s0_siblings1.push(
                qe[0]
                    .1
                    .iter()
                    .map(|e| e.iter().map(|ee| M::from_basefield(ee)).collect::<Vec<M::MTNode>>())
                    .collect::<Vec<Vec<M::MTNode>>>(),
            );

            s0_vals2.push(qe[1].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
            s0_siblings2.push(
                qe[1]
                    .1
                    .iter()
                    .map(|e| e.iter().map(|ee| M::from_basefield(ee)).collect::<Vec<M::MTNode>>())
                    .collect::<Vec<Vec<M::MTNode>>>(),
            );

            s0_vals3.push(qe[2].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
            s0_siblings3.push(
                qe[2]
                    .1
                    .iter()
                    .map(|e| e.iter().map(|ee| M::from_basefield(ee)).collect::<Vec<M::MTNode>>())
                    .collect::<Vec<Vec<M::MTNode>>>(),
            );

            s0_vals4.push(qe[3].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
            s0_siblings4.push(
                qe[3]
                    .1
                    .iter()
                    .map(|e| e.iter().map(|ee| M::from_basefield(ee)).collect::<Vec<M::MTNode>>())
                    .collect::<Vec<Vec<M::MTNode>>>(),
            );

            s0_valsC.push(qe[4].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
            s0_siblingsC.push(
                qe[4]
                    .1
                    .iter()
                    .map(|e| e.iter().map(|ee| M::from_basefield(ee)).collect::<Vec<M::MTNode>>())
                    .collect::<Vec<Vec<M::MTNode>>>(),
            );
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

        let source = TypeId::of::<<M::MTNode as MTNodeType>::BaseField>();
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
                let root: MT::MTNode =
                    serde_json::from_value(map.get("root1").unwrap().clone()).unwrap();
                sp.root1 = root;

                let root: MT::MTNode =
                    serde_json::from_value(map.get("root2").unwrap().clone()).unwrap();
                sp.root2 = root;

                let root: MT::MTNode =
                    serde_json::from_value(map.get("root3").unwrap().clone()).unwrap();
                sp.root3 = root;

                let root: MT::MTNode =
                    serde_json::from_value(map.get("root4").unwrap().clone()).unwrap();
                sp.root4 = root;

                let root = map.get("rootC");
                if root.is_some() {
                    let root: MT::MTNode = serde_json::from_value(root.unwrap().clone()).unwrap();
                    sp.rootC = Some(root);
                }

                let prover_addr = map.get("proverAddr");
                if prover_addr.is_some() {
                    sp.prover_addr = serde_json::from_value(prover_addr.unwrap().clone()).unwrap();
                }
                sp.evals = serde_json::from_value(map.get("evals").unwrap().clone()).unwrap();

                sp.publics = serde_json::from_value(map.get("publics").unwrap().clone()).unwrap();

                let mut fri_proof: FRIProof<MT::ExtendField, MT> = FRIProof::default();

                // search all s{i}_root keys, to avoid regex matching, we assume the max query is
                // less than 32
                let num_query: usize = (1..32)
                    .map(|i| {
                        let key = map.get(&format!("s{}_root", i));
                        if key.is_some() {
                            i
                        } else {
                            0
                        }
                    })
                    .max()
                    .unwrap();
                log::trace!("num_query: {}", num_query);

                fri_proof.queries = vec![Query::default(); num_query + 1];

                let mut s0_vals_all: Vec<Vec<Vec<FGL>>> = vec![];
                let mut s0_siblings_all: Vec<Vec<Vec<Vec<MT::MTNode>>>> = vec![];
                // handle queries[0]
                for j in ["1", "2", "3", "4", "C"] {
                    let key = map.get(&format!("s0_vals{}", j));
                    if key.is_none() {
                        log::info!("skip s0_vals{}", j);
                        s0_vals_all.push(vec![]);
                        s0_siblings_all.push(vec![]);
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

                    let key = map.get(&format!("s0_siblings{}", j));
                    let s0_siblings: Vec<Vec<Vec<MT::MTNode>>> =
                        serde_json::from_value(key.unwrap().clone()).unwrap();
                    s0_vals_all.push(s0_vals);
                    s0_siblings_all.push(s0_siblings);
                }

                // let qe = &self.fri_proof.queries[0].pol_queries;
                // s0_vals1: [qe[0][0].0, qe[1][0].0, .., qe[q][0].0]
                // s0_siblings1: [qe[0][0].1, qe[1][0].1, .., qe[q][0].1]
                // ...
                // s0_valsC: [qe[0][4].0, qe[1][4].0, .., qe[q][4].0]
                // s0_siblingsC: [qe[0][4].1, qe[1][4].1, .., qe[q][4].1]
                //
                // let mut s0_vals_all: Vec<Vec<Vec<FGL>>> = [s0_vals1, ..., s0_valsC]
                // let mut s0_siblings_all: Vec<Vec<Vec<Vec<<MT::MTNode as MTNodeType>::BaseField>>>> = [s0_siblings1, ..., s0_siblingsC]
                //
                // We have:
                // qe[i][k] = s0_vals_all[k][i], k in [0, 5)
                let num_pol_queries = s0_vals_all[0].len();
                fri_proof.queries[0].pol_queries = vec![vec![]; num_pol_queries];
                for i in 0..num_pol_queries {
                    fri_proof.queries[0].pol_queries[i] = vec![(vec![], vec![]); 5];
                    for k in 0..5 {
                        if s0_vals_all[k].len() < num_pol_queries {
                            log::trace!(
                                "resize {},{} from {} to {}",
                                i,
                                k,
                                s0_vals_all[k].len(),
                                num_pol_queries
                            );
                            s0_vals_all[k].resize_with(num_pol_queries, Vec::new);
                            s0_siblings_all[k].resize_with(num_pol_queries, Vec::new);
                        }
                        let node_to_bf =
                            crate::traits::mt_node_to_basefield::<MT>(&s0_siblings_all[k][i]);
                        fri_proof.queries[0].pol_queries[i][k] =
                            (s0_vals_all[k][i].clone(), node_to_bf);
                    }
                }

                // handle query 1 to num_query
                for i in 1..=num_query {
                    let key = map.get(&format!("s{}_root", i));
                    let root: MT::MTNode = serde_json::from_value(key.unwrap().clone()).unwrap();
                    fri_proof.queries[i].root = root;

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
                    let sibs: Vec<Vec<Vec<MT::MTNode>>> =
                        serde_json::from_value(key.unwrap().clone()).unwrap();
                    fri_proof.queries[i].pol_queries = vec![vec![]; num_pol_queries];
                    for q in 0..num_pol_queries {
                        let node_to_bf = crate::traits::mt_node_to_basefield::<MT>(&sibs[q]);
                        fri_proof.queries[i].pol_queries[q].push((vals[q].clone(), node_to_bf));
                    }
                }

                // handle finalPol
                let key = map.get("finalPol");
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
    use crate::merklehash::MerkleTreeGL;
    use crate::merklehash_bls12381::MerkleTreeBLS12381;
    use crate::merklehash_bn128::MerkleTreeBN128;
    use crate::polsarray::PolKind;
    use crate::polsarray::PolsArray;
    use crate::serializer::StarkProof;
    use crate::stark_setup::StarkSetup;
    use crate::traits::FieldExtension;
    use crate::traits::MTNodeType;
    use crate::transcript::TranscriptGL;
    use crate::transcript_bls12381::TranscriptBLS128;
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
        let input =
            F3G::from_vec([FGL::rand(&mut rng), FGL::rand(&mut rng), FGL::rand(&mut rng)].to_vec());
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
    fn test_serialize_element_digest() {
        env_logger::try_init().unwrap_or_default();
        let mut rng = rand::thread_rng();
        let four_fgl = ElementDigest::<4, FGL>::new(&[
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
            FGL::rand(&mut rng),
        ]);

        let four_fgl_ser = serde_json::to_string(&four_fgl).unwrap();
        let actual_four_fgl: ElementDigest<4, FGL> = serde_json::from_str(&four_fgl_ser).unwrap();
        assert_eq!(four_fgl.0, actual_four_fgl.0);

        let one_fgl: ElementDigest<4, FGL> =
            ElementDigest::<4, FGL>::new(&[FGL::rand(&mut rng), FGL::ZERO, FGL::ZERO, FGL::ZERO]);
        let one_fgl_ser = serde_json::to_string(&one_fgl).unwrap();
        let actual_one_fgl: ElementDigest<4, FGL> = serde_json::from_str(&one_fgl_ser).unwrap();
        assert_eq!(one_fgl.0, actual_one_fgl.0);

        let one_fr: ElementDigest<4, Fr> = ElementDigest::<4, Fr>::from_scalar(&Fr::rand(&mut rng));
        let one_fr_ser = serde_json::to_string(&one_fr).unwrap();
        let actual_one_fr: ElementDigest<4, Fr> = serde_json::from_str(&one_fr_ser).unwrap();
        assert_eq!(one_fr.0, actual_one_fr.0);

        let one_fr: ElementDigest<4, Fr_BLS12381> =
            ElementDigest::<4, Fr_BLS12381>::from_scalar(&Fr_BLS12381::rand(&mut rng));
        let one_fr_ser = serde_json::to_string(&one_fr).unwrap();
        let actual_one_fr: ElementDigest<4, Fr_BLS12381> =
            serde_json::from_str(&one_fr_ser).unwrap();
        assert_eq!(one_fr.0, actual_one_fr.0);
    }

    #[test]
    fn test_serialize_stark_proof_bn128_ser_der() {
        env_logger::try_init().unwrap_or_default();
        let mut pil = load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/fib.cm").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();

        let setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        //let fr_root: Fr = Fr(setup.const_root.as_scalar::<Fr>());

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
        let actual: StarkProof<MerkleTreeBN128> = serde_json::from_str(&serialized).unwrap();

        let mut file = File::create("/tmp/test_stark_proof_serialize.actual.json").unwrap();
        let serialized2 = serde_json::to_string(&actual).unwrap();
        write!(file, "{}", serialized2).unwrap();

        // assert
        assert_eq!(serialized, serialized2);
        assert_eq!(actual.root1, starkproof.root1);
        assert_eq!(actual.root2, starkproof.root2);
        assert_eq!(actual.root3, starkproof.root3);
        assert_eq!(actual.root4, starkproof.root4);
        assert_eq!(actual.rootC, starkproof.rootC);
        assert_eq!(actual.publics, starkproof.publics);
        assert_eq!(actual.evals, starkproof.evals);
        assert_eq!(actual.fri_proof, starkproof.fri_proof);
        assert_eq!(actual, starkproof);
    }

    #[test]
    fn test_serialize_stark_proof_gl_ser_der() {
        let mut pil = load_json::<PIL>("data/fib.pil.json.gl").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const.gl").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/fib.cm.gl").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json.gl").unwrap();

        let setup =
            StarkSetup::<MerkleTreeGL>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        //let fr_root: Fr = Fr(setup.const_root.as_scalar::<Fr>());

        let starkproof = StarkProof::<MerkleTreeGL>::stark_gen::<TranscriptGL>(
            cm_pol,
            const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
            "",
        )
        .unwrap();

        // serde to json
        let serialized = serde_json::to_string(&starkproof).unwrap();
        let mut file = File::create("/tmp/test_stark_proof_serialize.gl.json").unwrap();
        write!(file, "{}", serialized).unwrap();
        // deserialized
        let actual: StarkProof<MerkleTreeGL> = serde_json::from_str(&serialized).unwrap();

        let mut file = File::create("/tmp/test_stark_proof_serialize.actual.gl.json").unwrap();
        let serialized2 = serde_json::to_string(&actual).unwrap();
        write!(file, "{}", serialized2).unwrap();

        // assert
        assert_eq!(serialized, serialized2);
        assert_eq!(actual.root1, starkproof.root1);
        assert_eq!(actual.root2, starkproof.root2);
        assert_eq!(actual.root3, starkproof.root3);
        assert_eq!(actual.root4, starkproof.root4);
        assert_eq!(actual.rootC, starkproof.rootC);
        assert_eq!(actual.publics, starkproof.publics);
        assert_eq!(actual.evals, starkproof.evals);
        assert_eq!(actual.fri_proof, starkproof.fri_proof);
        assert_eq!(actual, starkproof);
    }

    #[test]
    fn test_serialize_stark_proof_bls12381_ser_der() {
        env_logger::try_init().unwrap_or_default();
        let mut pil = load_json::<PIL>("data/fib.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/fib.const").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/fib.cm").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json.bls12381").unwrap();

        let setup =
            StarkSetup::<MerkleTreeBLS12381>::new(&const_pol, &mut pil, &stark_struct, None)
                .unwrap();
        //let fr_root: Fr = Fr(setup.const_root.as_scalar::<Fr>());

        let starkproof = StarkProof::<MerkleTreeBLS12381>::stark_gen::<TranscriptBLS128>(
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
        let mut file = File::create("/tmp/test_stark_proof_serialize.bls12381.json").unwrap();
        write!(file, "{}", serialized).unwrap();
        // deserialized
        let actual: StarkProof<MerkleTreeBLS12381> = serde_json::from_str(&serialized).unwrap();

        let mut file =
            File::create("/tmp/test_stark_proof_serialize.bls12381.actual.json").unwrap();
        let serialized2 = serde_json::to_string(&actual).unwrap();
        write!(file, "{}", serialized2).unwrap();

        // assert
        assert_eq!(serialized, serialized2);
        assert_eq!(actual.root1, starkproof.root1);
        assert_eq!(actual.root2, starkproof.root2);
        assert_eq!(actual.root3, starkproof.root3);
        assert_eq!(actual.root4, starkproof.root4);
        assert_eq!(actual.rootC, starkproof.rootC);
        assert_eq!(actual.publics, starkproof.publics);
        assert_eq!(actual.evals, starkproof.evals);
        assert_eq!(actual.fri_proof, starkproof.fri_proof);
        assert_eq!(actual, starkproof);
    }
}
