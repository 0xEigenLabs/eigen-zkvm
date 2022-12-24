use crate::errors::Result;
use crate::poseidon_opt::Poseidon;
use std::collections::VecDeque;

use crate::f3g::F3G;
use winter_math::fields::f64::BaseElement;

use num_bigint::BigUint;

pub struct Transcript {
    state: [BaseElement; 4],
    poseidon: Poseidon,
    pending: Vec<BaseElement>,
    out: VecDeque<BaseElement>,
}

impl TranscriptBN128 {
    pub fn new() -> Self {
        Self {
            state: [BaseElement::ZERO; 4],
            poseidon: Poseidon::new(),
            pending: Vec::new(),
            out: VecDeque::new(),
        }
    }

    pub fn get_field(&mut self) -> F3G {
        let a = self.get_fields1().unwrap();
        let b = self.get_fields1().unwrap();
        let c = self.get_fields1().unwrap();
        F3G::new(a, b, c)
    }

    pub fn get_fields1(&mut self) -> Result<BaseElement> {
        if self.out.len() > 0 {
            return Ok(self.out.pop_front().unwrap());
        }
        self.update_state()?;
        self.get_fields1()
    }

    fn update_state(&mut self) -> Result<()> {
        while self.pending.len() < 8 {
            self.pending.push(Fr::zero());
        }
        self.out = VecDeque::from(self.poseidon.hash_ex(&self.pending, &self.state, 12)?);
        self.pending = vec![];
        self.state.copy_from_slice(&self.out[0..4]);
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
        if self.pending.len() == 8 {
            self.update_state()?;
        }
        Ok(())
    }

    pub fn get_permutations(&mut self, n: usize, nbits: usize) -> Result<Vec<usize>> {
        let total_bits = n * nbits;
        let n_fields = (total_bits - 1) / 63 + 1;
        let mut fields: Vec<BigUint> = Vec::new();
        for _i in 0..n_fields {
            fields.push(self.get_fields1());
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
                    a = a + (1 << j);
                }
                cur_bit += 1;
                if cur_bit == 63 {
                    cur_bit = 0;
                    cur_field += 1;
                }
            }
            res.push(a);
        }
        Ok(res)
    }
}
