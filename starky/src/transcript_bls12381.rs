#![allow(dead_code)]
use crate::digest::ElementDigest;
use crate::ff::Field;
use crate::field_bls12381::{Fr, FrRepr};
use crate::helper::{biguint_to_be, fr_bls12381_to_biguint};
use crate::poseidon_bls12381_opt::Poseidon;
use crate::traits::Transcript;
use crate::traits::{FieldExtension, MTNodeType};
use anyhow::Result;
use ff::*;
use fields::field_gl::Fr as FGL;
use num_bigint::BigUint;
use std::collections::VecDeque;

pub struct TranscriptBLS128 {
    state: Fr,
    poseidon: Poseidon,
    pending: Vec<Fr>,
    out: VecDeque<Fr>,
    out3: VecDeque<FGL>,
}

impl TranscriptBLS128 {
    fn update_state(&mut self) -> Result<()> {
        while self.pending.len() < 16 {
            self.pending.push(Fr::zero());
        }
        self.out = VecDeque::from(self.poseidon.hash_ex(&self.pending, &self.state, 17)?);
        self.out3 = VecDeque::new();
        self.pending = vec![];
        self.state = self.out[0];
        Ok(())
    }
    fn add_1(&mut self, e: &Fr) -> Result<()> {
        self.out = VecDeque::new();
        log::trace!("add_1: {:?}", fr_bls12381_to_biguint(e));
        self.pending.push(*e);
        if self.pending.len() == 16 {
            self.update_state()?;
        }
        Ok(())
    }

    fn get_fields253(&mut self) -> Result<Fr> {
        if !self.out.is_empty() {
            return Ok(self.out.pop_front().unwrap());
        }
        self.update_state()?;
        self.get_fields253()
    }
}

impl Transcript for TranscriptBLS128 {
    fn new() -> Self {
        Self {
            state: Fr::zero(),
            poseidon: Poseidon::new(),
            pending: Vec::new(),
            out: VecDeque::new(),
            out3: VecDeque::new(),
        }
    }

    fn get_field<F: FieldExtension>(&mut self) -> F {
        let a = self.get_fields1().unwrap();
        let b = self.get_fields1().unwrap();
        let c = self.get_fields1().unwrap();
        F::from_vec(vec![a, b, c])
    }

    fn get_fields1(&mut self) -> Result<FGL> {
        if !self.out3.is_empty() {
            log::trace!("get_fields1 {},", self.out3[0]);
            return Ok(self.out3.pop_front().unwrap());
        }

        if !self.out.is_empty() {
            let v = self.out.pop_front().unwrap();
            let bv = fr_bls12381_to_biguint(&v);
            let mask = BigUint::from(0xFFFFFFFFFFFFFFFFu128);
            self.out3.push_back(biguint_to_be(&(&bv & &mask)));
            self.out3.push_back(biguint_to_be(&((&bv >> 64) & &mask))); //FIXME: optimization
            self.out3.push_back(biguint_to_be(&((&bv >> 128) & &mask)));
            return self.get_fields1();
        }
        self.update_state()?;
        self.get_fields1()
    }

    fn put(&mut self, es: &[Vec<FGL>]) -> Result<()> {
        for e in es.iter() {
            let e: Fr = match e.len() {
                1 => Fr::from_repr(FrRepr::from(e[0].as_int())).unwrap(),
                4 => {
                    let ie: ElementDigest<4> = ElementDigest::new(&[e[0], e[1], e[2], e[3]]);
                    Fr(ie.as_scalar::<Fr>())
                }
                _ => panic!("Invalid elements as inputs to transcript"),
            };
            self.add_1(&e)?;
        }
        Ok(())
    }

    fn get_permutations(&mut self, n: usize, nbits: usize) -> Result<Vec<usize>> {
        let total_bits = n * nbits;
        let n_fields = (total_bits - 1) / 253 + 1;
        let mut fields: Vec<BigUint> = Vec::new();
        for _i in 0..n_fields {
            fields.push(fr_bls12381_to_biguint(&self.get_fields253()?));
        }
        let mut res: Vec<usize> = vec![];
        let mut cur_field = 0;
        let mut cur_bit = 0usize;
        let one = BigUint::from(1u32);
        for _i in 0..n {
            let mut a = 0usize;
            for j in 0..nbits {
                let shift = &fields[cur_field] >> cur_bit;
                let bit = shift & &one;
                if bit == one {
                    a += 1 << j;
                }
                cur_bit += 1;
                if cur_bit == 253 {
                    cur_bit = 0;
                    cur_field += 1;
                }
            }
            res.push(a);
        }
        Ok(res)
    }
}
