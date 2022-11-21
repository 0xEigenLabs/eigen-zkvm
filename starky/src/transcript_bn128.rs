use crate::errors::Result;
use crate::helper::{biguint_to_be, biguint_to_fr, fr_to_biguint};
use crate::poseidon_bn128::{Fr, Poseidon};
use ff::*;
use std::collections::VecDeque;

use winter_math::fields::f64::BaseElement;

use num_bigint::BigUint;
use num_traits::Num;
use num_traits::ToPrimitive;

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

    pub fn get_field(&mut self) -> [BaseElement; 3] {
        let mut res: [BaseElement; 3] = [
            self.get_fields1().unwrap(),
            self.get_fields1().unwrap(),
            self.get_fields1().unwrap(),
        ];

        res
    }

    pub fn get_fields1(&mut self) -> Result<BaseElement> {
        if self.out3.len() > 0 {
            return Ok(self.out3.pop_front().unwrap());
        }

        if self.out.len() > 0 {
            let v = self.out.pop_front().unwrap();
            let bv = fr_to_biguint(&v);
            let mask = BigUint::from(0xFFFFFFFFFFFFFFFFu128);
            self.out3[0] = biguint_to_be(&(&bv & &mask));
            self.out3[1] = biguint_to_be(&((&bv >> 64) & &mask));
            self.out3[2] = biguint_to_be(&((&bv >> 128) & &mask));
            return self.get_fields1();
        }
        self.update_state()?;
        self.get_fields1()
    }

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

    pub fn put(&mut self, es: &[Fr]) -> Result<()> {
        for e in es.iter() {
            self.add_1(e)?;
        }
        Ok(())
    }

    fn add_1(&mut self, e: &Fr) -> Result<()> {
        self.out = VecDeque::new();
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

    fn get_permutations(&mut self, n: usize, nbits: usize) -> Result<Vec<Fr>> {
        let total_bits = n * nbits;
        let NFields = (total_bits - 1) / 253 + 1;
        let mut fields: Vec<BigUint> = Vec::new();
        for i in 0..NFields {
            fields.push(fr_to_biguint(&self.get_fields253()?));
        }
        let mut res: Vec<Fr> = vec![];
        let mut cur_field = 0;
        let mut cur_bit = 0usize;
        let one = BigUint::from(1u32);
        for i in 0..n {
            let mut a = BigUint::from(0u32);
            for j in 0..nbits {
                let shift = &fields[cur_field] >> cur_bit;
                let bit = shift & &one;
                if bit == one {
                    a = a + BigUint::from(1u128 << j);
                }
                cur_bit += 1;
                if cur_bit == 253 {
                    cur_bit = 0;
                    cur_field += 1;
                }
            }
            res.push(biguint_to_fr(&a));
        }
        Ok(res)
    }
}
