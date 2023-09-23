// input json of plonk
#![allow(non_snake_case)]
use crate::f3g::F3G;
use crate::field_bn128::Fr;
use crate::field_bls12381::Fr as Fr_bls12381;
use crate::helper;
use crate::stark_gen::StarkProof;
use crate::traits::{MTNodeType, MerkleTree};
use plonky::field_gl::Fr as FGL;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

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

pub struct Input<T: MTNodeType>(T, String);

impl<T: MTNodeType> Input<T> {
    pub fn new(e: T, hashtype: String) -> Self {
        Input(e, hashtype)
    }
    pub fn is_dim_1(&self) -> bool {
        let e = self.0.as_elements();
        e[1] == e[2] && e[1] == e[3] && e[1] == FGL::ZERO
    }
}

impl<T: MTNodeType + Clone> Serialize for Input<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let e = self.0.as_elements();
        if self.is_dim_1() {
            return serializer.serialize_str(&e[0].as_int().to_string());
        }
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
                let mut seq = serializer.serialize_seq(Some(4))?;
                for v in e.iter() {
                    seq.serialize_element(&v.as_int().to_string())?;
                }
                seq.end()
            }
            _ => panic!("Invalid hashtype {}", self.1),
        }
    }
}

impl<T: MTNodeType> From<Fr> for Input<T> {
    fn from(val: Fr) -> Self {
        let e = T::from_scalar(&val);
        Self(e, "".to_string())
    }
}

impl<T: MTNodeType> From<Fr_bls12381> for Input<T> {
    fn from(val: Fr_bls12381) -> Self {
        let e = T::from_scalar(&val);
        Self(e, "".to_string())
    }
}

impl<T: MTNodeType> From<FGL> for Input<T> {
    fn from(val: FGL) -> Self {
        Self(
            T::new(&[val, FGL::ZERO, FGL::ZERO, FGL::ZERO]),
            "".to_string(),
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
                    &Input::<M::MTNode>::new(value.clone(), hashtype.clone()),
                )?;
            }
            None => {}
        }
        map.serialize_entry(
            "root1",
            &Input::<M::MTNode>::new(self.root1, hashtype.clone()),
        )?;
        map.serialize_entry(
            "root2",
            &Input::<M::MTNode>::new(self.root2, hashtype.clone()),
        )?;
        map.serialize_entry(
            "root3",
            &Input::<M::MTNode>::new(self.root3, hashtype.clone()),
        )?;
        map.serialize_entry(
            "root4",
            &Input::<M::MTNode>::new(self.root4, hashtype.clone()),
        )?;
        map.serialize_entry("evals", &self.evals)?;

        for i in 1..(self.fri_proof.queries.len()) {
            map.serialize_entry(
                &format!("s{}_root", i),
                &Input::new(self.fri_proof.queries[i].root, hashtype.clone()),
            )?;
            let mut vals: Vec<Vec<F3G>> = vec![];
            let mut sibs: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
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
                                    let mut res: Input<M::MTNode> = ee.clone().into();
                                    res.1 = hashtype.clone();
                                    res
                                })
                                .collect::<Vec<Input<M::MTNode>>>()
                        })
                        .collect::<Vec<Vec<Input<M::MTNode>>>>(),
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
        let mut s0_siblings1: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
        let mut s0_siblings2: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
        let mut s0_siblings3: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
        let mut s0_siblings4: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];
        let mut s0_siblingsC: Vec<Vec<Vec<Input<M::MTNode>>>> = vec![];

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
                                let mut res: Input<M::MTNode> = ee.clone().into();
                                res.1 = hashtype.clone();
                                res
                            })
                            .collect::<Vec<Input<M::MTNode>>>()
                    })
                    .collect::<Vec<Vec<Input<M::MTNode>>>>(),
            );

            if qe[1].0.len() > 0 {
                s0_vals2.push(qe[1].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
                s0_siblings2.push(
                    qe[1]
                        .1
                        .iter()
                        .map(|e| {
                            e.iter()
                                .map(|ee| {
                                    let mut res: Input<M::MTNode> = ee.clone().into();
                                    res.1 = hashtype.clone();
                                    res
                                })
                                .collect::<Vec<Input<M::MTNode>>>()
                        })
                        .collect::<Vec<Vec<Input<M::MTNode>>>>(),
                );
            }

            if qe[2].0.len() > 0 {
                s0_vals3.push(qe[2].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
                s0_siblings3.push(
                    qe[2]
                        .1
                        .iter()
                        .map(|e| {
                            e.iter()
                                .map(|ee| {
                                    let mut res: Input<M::MTNode> = ee.clone().into();
                                    res.1 = hashtype.clone();
                                    res
                                })
                                .collect::<Vec<Input<M::MTNode>>>()
                        })
                        .collect::<Vec<Vec<Input<M::MTNode>>>>(),
                );
            }

            let qe = &self.fri_proof.queries[0].pol_queries[i];
            if qe[3].0.len() > 0 {
                s0_vals4.push(qe[3].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
                s0_siblings4.push(
                    qe[3]
                        .1
                        .iter()
                        .map(|e| {
                            e.iter()
                                .map(|ee| {
                                    let mut res: Input<M::MTNode> = ee.clone().into();
                                    res.1 = hashtype.clone();
                                    res
                                })
                                .collect::<Vec<Input<M::MTNode>>>()
                        })
                        .collect::<Vec<Vec<Input<M::MTNode>>>>(),
                );
            }

            if qe[4].0.len() > 0 {
                s0_valsC.push(qe[4].0.iter().map(|e| F3G::from(*e)).collect::<Vec<F3G>>());
                s0_siblingsC.push(
                    qe[4]
                        .1
                        .iter()
                        .map(|e| {
                            e.iter()
                                .map(|ee| {
                                    let mut res: Input<M::MTNode> = ee.clone().into();
                                    res.1 = hashtype.clone();
                                    res
                                })
                                .collect::<Vec<Input<M::MTNode>>>()
                        })
                        .collect::<Vec<Vec<Input<M::MTNode>>>>(),
                );
            }
        }

        map.serialize_entry("s0_vals1", &s0_vals1)?;
        if s0_vals2.len() > 0 {
            map.serialize_entry("s0_vals2", &s0_vals2)?;
        }
        if s0_vals3.len() > 0 {
            map.serialize_entry("s0_vals3", &s0_vals3)?;
        }
        map.serialize_entry("s0_vals4", &s0_vals4)?;
        map.serialize_entry("s0_valsC", &s0_valsC)?;
        map.serialize_entry("s0_siblings1", &s0_siblings1)?;
        if s0_siblings2.len() > 0 {
            map.serialize_entry("s0_siblings2", &s0_siblings2)?;
        }
        if s0_siblings3.len() > 0 {
            map.serialize_entry("s0_siblings3", &s0_siblings3)?;
        }
        map.serialize_entry("s0_siblings4", &s0_siblings4)?;
        map.serialize_entry("s0_siblingsC", &s0_siblingsC)?;
        map.serialize_entry("finalPol", &self.fri_proof.last)?;
        map.serialize_entry("publics", &self.publics)?;
        if hashtype.as_str() == "BN128" {
            map.serialize_entry(
                "proverAddr",
                "273030697313060285579891744179749754319274977764",
            )?;
        }
        map.end()
    }
}
