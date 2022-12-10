#![allow(dead_code)]
use crate::errors::Result;
use crate::field_bn128::Fr;
use crate::helper::{biguint_to_be, fr_to_biguint};
use crate::poseidon_bn128_opt::Poseidon;
use ff::*;
use std::collections::VecDeque;

use crate::f3g::F3G;
use winter_math::fields::f64::BaseElement;

use num_bigint::BigUint;

pub struct TranscriptBN128 {
    state: Fr,
    poseidon: Poseidon,
    pending: Vec<Fr>,
    out: VecDeque<Fr>,
    out3: VecDeque<BaseElement>,
}

impl TranscriptBN128 {
    pub fn new() -> Self {
        Self {
            state: Fr::zero(),
            poseidon: Poseidon::new(),
            pending: Vec::new(),
            out: VecDeque::new(),
            out3: VecDeque::new(),
        }
    }

    pub fn get_field(&mut self) -> F3G {
        let a = self.get_fields1().unwrap();
        let b = self.get_fields1().unwrap();
        let c = self.get_fields1().unwrap();
        F3G::new(a, b, c)
    }

    pub fn get_fields1(&mut self) -> Result<BaseElement> {
        if self.out3.len() > 0 {
            //println!("get_fields1 {},", self.out3[0]);
            return Ok(self.out3.pop_front().unwrap());
        }

        if self.out.len() > 0 {
            let v = self.out.pop_front().unwrap();
            //println!("get_fields1 out3 v={}", v);
            let bv = fr_to_biguint(&v);
            //println!("get_fields1 out3 {}", bv);
            let mask = BigUint::from(0xFFFFFFFFFFFFFFFFu128);
            self.out3.push_back(biguint_to_be(&(&bv & &mask)));
            self.out3.push_back(biguint_to_be(&((&bv >> 64) & &mask))); //FIXME: optimization
            self.out3.push_back(biguint_to_be(&((&bv >> 128) & &mask)));
            return self.get_fields1();
        }
        self.update_state()?;
        self.get_fields1()
    }

    fn update_state(&mut self) -> Result<()> {
        while self.pending.len() < 16 {
            self.pending.push(Fr::zero());
        }
        //for i in self.pending.iter() {
        //    println!("update_state: {}", crate::helper::fr_to_biguint(i));
        //    println!(
        //        "update_state: MONT {}",
        //        Fr::from_repr(i.into_raw_repr()).unwrap()
        //    );
        //}
        //println!("self.state: {}", crate::helper::fr_to_biguint(&self.state));
        self.out = VecDeque::from(self.poseidon.hash_ex(&self.pending, &self.state, 17)?);
        self.out3 = VecDeque::new();
        self.pending = vec![];
        self.state = self.out[0];
        Ok(())
    }

    pub fn put(&mut self, es: &[Fr]) -> Result<()> {
        for e in es.iter() {
            self.add_1(e)?;
        }
        Ok(())
    }

    fn add_1(&mut self, e: &Fr) -> Result<()> {
        self.out = VecDeque::new();
        //println!("add_1 to pending: {:?}", fr_to_biguint(e));
        self.pending.push(e.clone());
        if self.pending.len() == 16 {
            self.update_state()?;
        }
        Ok(())
    }

    fn get_fields253(&mut self) -> Result<Fr> {
        if self.out.len() > 0 {
            return Ok(self.out.pop_front().unwrap());
        }
        self.update_state()?;
        self.get_fields253()
    }

    pub fn get_permutations(&mut self, n: usize, nbits: usize) -> Result<Vec<usize>> {
        let total_bits = n * nbits;
        let n_fields = (total_bits - 1) / 253 + 1;
        let mut fields: Vec<BigUint> = Vec::new();
        for _i in 0..n_fields {
            fields.push(fr_to_biguint(&self.get_fields253()?));
        }
        //println!("get_permutations: {:?}", fields);
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
                    a = a + (1 << j);
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
