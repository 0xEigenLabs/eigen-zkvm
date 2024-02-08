use crate::f3g::F3G;
use crate::f5g::F5G;
use crate::field_bls12381::Fr as Fr_bls12381;
use crate::field_bn128::Fr;
use crate::fri::{FRIProof, Query};
use crate::helper;
use crate::serializer::Input;
use crate::stark_gen::StarkProof;
use crate::traits::FieldExtension;
use crate::traits::{MTNodeType, MerkleTree};
use crate::types::StarkStruct;
use fields::field_gl::Fr as FGL;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use std::fmt;
use std::marker::PhantomData;

// A Visitor is a type that holds methods that a Deserializer can drive
// depending on what is contained in the input data.
//
// In the case of a map we need generic type parameters K and V to be
// able to set the output type correctly, but don't require any state.
// This is an example of a "zero sized type" in Rust. The PhantomData
// keeps the compiler from complaining about unused generic type
// parameters.
struct StarkProofVisitor<M: MerkleTree> {
    marker: PhantomData<fn() -> StarkProof<M>>,
}

impl<M: MerkleTree> StarkProofVisitor<M> {
    fn new() -> Self {
        StarkProofVisitor {
            marker: PhantomData,
        }
    }
}

// This is the trait that Deserializers are going to be driving. There
// is one method for each type of data that our type knows how to
// deserialize from. There are many other methods that are not
// implemented here, for example deserializing from integers or strings.
// By default those methods will return an error, which makes sense
// because we cannot deserialize a StarkProof from an integer or string.
impl<'de, M: MerkleTree> Visitor<'de> for StarkProofVisitor<M>
where
    M: Deserialize<'de>,
{
    // The type that our Visitor is going to produce.
    type Value = StarkProof<M>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a very special map")
    }

    fn visit_map(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        // While there are entries remaining in the input, add them
        // into our map.
        let map_len = map.size_hint().unwrap_or(0);
        // root, evals, friProof * 3, s0_val{1,2,3,4,C},  s0_siblings{1,2,3,4,C}, finalPol
        let fri_proof_queries_len = (map_len - 16) / 3 + 1;
        let mut fri_proof_pol_queries_len = 0; // obtain is from starkproof.json

        let mut stark_proof = StarkProof::default();
        let stark_struct = StarkStruct::default();
        // let fri_proof = FRIProof::<M::ExtendField, M>::default();

        while let Some(key) = map.next_key()? {
            // match some fields;
            match key {
                // Mark: We regard map as stack(LIFO)..
                "fri_proof_queries_len" => {
                    let queries_len = map.next_value()?;
                    assert_eq!(queries_len, fri_proof_queries_len);
                }
                "fri_proof_pol_queries_len" => {
                    fri_proof_pol_queries_len = map.next_value()?;
                }
                "rootC" => {
                    let input: Input<M::MTNode> = map.next_value()?;
                    stark_proof.rootC = input.0;
                }
                "root1" => {
                    let input: Input<M::MTNode> = map.next_value()?;
                    // pub struct Input<T: MTNodeType>(T, String);
                    stark_proof.root1 = input.0;
                    // append HashType here.
                    stark_struct.verificationHashType = input.1;
                }
                "root2" => {
                    let input: Input<M::MTNode> = map.next_value()?;
                    stark_proof.root2 = input.0;
                }
                "root3" => {
                    let input: Input<M::MTNode> = map.next_value()?;
                    stark_proof.root3 = input.0;
                }
                "root4" => {
                    let input: Input<M::MTNode> = map.next_value()?;
                    stark_proof.root4 = input.0;
                }
                "evals" => {
                    stark_proof.evals = Some(map.next_value()?);
                } // "s0_vals1" => {
                  //     let s0_vals1: Vec<Vec<F3G>> = map.next_value()?;
                  //     // TODO
                  //     // stark_proof.evals = Some(map.next_value()?);
                  // }
                  // "s0_vals2" => {
                  //     let s0_vals2: Vec<Vec<F3G>> = map.next_value()?;
                  //     // #[derive(Debug, Clone, Default)]
                  //     // pub struct FRIProof<F: FieldExtension, M: MerkleTree<ExtendField = F>> {
                  //     //     pub queries: Vec<Query<M::BaseField, M::MTNode>>,
                  //     //     pub last: Vec<F>,
                  //     // }
                  //     // stark_proof.evals = Some(map.next_value()?);
                  // }
                  // "s0_vals3" => {
                  //     let s0_vals3: Vec<Vec<F3G>> = map.next_value()?;
                  //
                  //     // stark_proof.evals = Some(map.next_value()?);
                  // }
                  // "s0_vals4" => {
                  //     let s0_vals4: Vec<Vec<F3G>> = map.next_value()?;
                  //
                  //     // stark_proof.evals = Some(map.next_value()?);
                  // }
                  // "s0_valsC" => {
                  //     let s0_valsC: Vec<Vec<F3G>> = map.next_value()?;
                  //
                  //     // stark_proof.evals = Some(map.next_value()?);
                  // }
                  // "s0_siblings1" => {
                  //     let s0_siblings1: Vec<Vec<Vec<Input<M::MTNode>>>> = map.next_value()?;
                  //
                  //     // stark_proof.evals = Some(map.next_value()?);
                  // }
                  // "s0_siblings2" => {
                  //     let s0_siblings2: Vec<Vec<Vec<Input<M::MTNode>>>> = map.next_value()?;
                  //
                  //     // stark_proof.evals = Some(map.next_value()?);
                  // }
                  // "s0_siblings3" => {
                  //     let s0_siblings3: Vec<Vec<Vec<Input<M::MTNode>>>> = map.next_value()?;
                  //
                  //     // stark_proof.evals = Some(map.next_value()?);
                  // }
                  // "s0_siblings4" => {
                  //     let s0_siblings4: Vec<Vec<Vec<Input<M::MTNode>>>> = map.next_value()?;
                  //
                  //     // stark_proof.evals = Some(map.next_value()?);
                  // }
                  // "s0_siblingsC" => {
                  //     let s0_siblingsC: Vec<Vec<Vec<Input<M::MTNode>>>> = map.next_value()?;
                  //
                  //     // stark_proof.evals = Some(map.next_value()?);
                  // }
            }

            // To detect poly ones;
            if key.contains("_root") {}
        }

        // TODO: Below are the serialized one.
        // for i in 1..(self.fri_proof.queries.len()) {
        //     map.serialize_entry(
        //         &format!("s{}_root", i),
        //         &Input::new(self.fri_proof.queries[i].root, hashtype.clone()),
        //     )?;
        //     let mut vals: Vec<Vec<F3G>> = vec![];
        //     let mut sibs: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
        //     for q in 0..self.fri_proof.queries[0].pol_queries.len() {
        //         let qe = &self.fri_proof.queries[i].pol_queries[q];
        //         vals.push(qe[0].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
        //         sibs.push(
        //             qe[0]
        //                 .1
        //                 .iter()
        //                 .map(|e| {
        //                     e.iter()
        //                         .map(|ee| {
        //                             let mut res: Input<M::MTNode> = ee.clone().into();
        //                             res.1 = hashtype.clone();
        //                             res
        //                         })
        //                         .collect::<Vec<Input<M::MTNode>>>()
        //                 })
        //                 .collect::<Vec<Vec<Input<M::MTNode>>>>(),
        //         );
        //     }
        //     map.serialize_entry(&format!("s{}_vals", i), &vals)?;
        //     map.serialize_entry(&format!("s{}_siblings", i), &sibs)?;
        // }
        //
        // let mut s0_vals1: Vec<Vec<F3G>> = vec![];
        // let mut s0_vals2: Vec<Vec<F3G>> = vec![];
        // let mut s0_vals3: Vec<Vec<F3G>> = vec![];
        // let mut s0_vals4: Vec<Vec<F3G>> = vec![];
        // let mut s0_valsC: Vec<Vec<F3G>> = vec![];
        // let mut s0_siblings1: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
        // let mut s0_siblings2: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
        // let mut s0_siblings3: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
        // let mut s0_siblings4: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
        // let mut s0_siblingsC: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
        //
        // for i in 0..self.fri_proof.queries[0].pol_queries.len() {
        //     //(leaf, path) represents each query
        //     let qe = &self.fri_proof.queries[0].pol_queries[i];
        //     s0_vals1.push(qe[0].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
        //     s0_siblings1.push(
        //         qe[0]
        //             .1
        //             .iter()
        //             .map(|e| {
        //                 e.iter()
        //                     .map(|ee| {
        //                         let mut res: Input<M::MTNode> = ee.clone().into();
        //                         res.1 = hashtype.clone();
        //                         res
        //                     })
        //                     .collect::<Vec<Input<M::MTNode>>>()
        //             })
        //             .collect::<Vec<Vec<Input<M::MTNode>>>>(),
        //     );
        //
        //     if !qe[1].0.is_empty() {
        //         s0_vals2.push(qe[1].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
        //         s0_siblings2.push(
        //             qe[1]
        //                 .1
        //                 .iter()
        //                 .map(|e| {
        //                     e.iter()
        //                         .map(|ee| {
        //                             let mut res: Input<M::MTNode> = ee.clone().into();
        //                             res.1 = hashtype.clone();
        //                             res
        //                         })
        //                         .collect::<Vec<Input<M::MTNode>>>()
        //                 })
        //                 .collect::<Vec<Vec<Input<M::MTNode>>>>(),
        //         );
        //     }
        //
        //     if !qe[2].0.is_empty() {
        //         s0_vals3.push(qe[2].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
        //         s0_siblings3.push(
        //             qe[2]
        //                 .1
        //                 .iter()
        //                 .map(|e| {
        //                     e.iter()
        //                         .map(|ee| {
        //                             let mut res: Input<M::MTNode> = ee.clone().into();
        //                             res.1 = hashtype.clone();
        //                             res
        //                         })
        //                         .collect::<Vec<Input<M::MTNode>>>()
        //                 })
        //                 .collect::<Vec<Vec<Input<M::MTNode>>>>(),
        //         );
        //     }
        //
        //     let qe = &self.fri_proof.queries[0].pol_queries[i];
        //     if !qe[3].0.is_empty() {
        //         s0_vals4.push(qe[3].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
        //         s0_siblings4.push(
        //             qe[3]
        //                 .1
        //                 .iter()
        //                 .map(|e| {
        //                     e.iter()
        //                         .map(|ee| {
        //                             let mut res: Input<M::MTNode> = ee.clone().into();
        //                             res.1 = hashtype.clone();
        //                             res
        //                         })
        //                         .collect::<Vec<Input<M::MTNode>>>()
        //                 })
        //                 .collect::<Vec<Vec<Input<M::MTNode>>>>(),
        //         );
        //     }
        //
        //     if !qe[4].0.is_empty() {
        //         s0_valsC.push(qe[4].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
        //         s0_siblingsC.push(
        //             qe[4]
        //                 .1
        //                 .iter()
        //                 .map(|e| {
        //                     e.iter()
        //                         .map(|ee| {
        //                             let mut res: Input<M::MTNode> = ee.clone().into();
        //                             res.1 = hashtype.clone();
        //                             res
        //                         })
        //                         .collect::<Vec<Input<M::MTNode>>>()
        //                 })
        //                 .collect::<Vec<Vec<Input<M::MTNode>>>>(),
        //         );
        //     }
        // }
        //
        // map.serialize_entry("s0_vals1", &s0_vals1)?;
        // if !s0_vals2.is_empty() {
        //     map.serialize_entry("s0_vals2", &s0_vals2)?;
        // }
        // if !s0_vals3.is_empty() {
        //     map.serialize_entry("s0_vals3", &s0_vals3)?;
        // }
        // map.serialize_entry("s0_vals4", &s0_vals4)?;
        // map.serialize_entry("s0_valsC", &s0_valsC)?;
        // map.serialize_entry("s0_siblings1", &s0_siblings1)?;
        // if !s0_siblings2.is_empty() {
        //     map.serialize_entry("s0_siblings2", &s0_siblings2)?;
        // }
        // if !s0_siblings3.is_empty() {
        //     map.serialize_entry("s0_siblings3", &s0_siblings3)?;
        // }
        // map.serialize_entry("s0_siblings4", &s0_siblings4)?;
        // map.serialize_entry("s0_siblingsC", &s0_siblingsC)?;
        // map.serialize_entry("finalPol", &self.fri_proof.last)?;
        // map.serialize_entry("publics", &self.publics)?;
        // if hashtype.as_str() == "BN128" || hashtype.as_str() == "BLS12381" {
        //     map.serialize_entry("proverAddr", &self.prover_addr)?;
        // }
        // map.end();

        Ok(stark_proof)
    }
}

// This is the trait that informs Serde how to deserialize StarkProof.
impl<'de, M> Deserialize<'de> for StarkProof<M>
where
    M: Deserialize<'de> + MerkleTree,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of StarkProof.
        deserializer.deserialize_map(StarkProofVisitor::new())
    }
}
