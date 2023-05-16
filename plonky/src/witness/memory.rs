//! Safe-ish interface for reading and writing specific types to the WASM runtime's memory,
//! modified from ark-circom
use crate::bellman_ce::{Field, PrimeField, PrimeFieldRepr, ScalarEngine};
use crate::errors::{EigenError, Result};
use crate::to_hex;
use num_bigint::{BigInt, BigUint};
use num_traits::Num;
use num_traits::ToPrimitive;
use num_traits::Zero;
use std::str::FromStr;
use std::{convert::TryFrom, ops::Deref};
use wasmer::{Memory, MemoryView};

#[derive(Clone, Debug)]
pub struct SafeMemory {
    pub memory: Memory,
    pub prime: BigInt,

    short_max: BigInt,
    short_min: BigInt,
    r_inv: BigInt,
    n32: usize,
}

impl Deref for SafeMemory {
    type Target = Memory;

    fn deref(&self) -> &Self::Target {
        &self.memory
    }
}

impl SafeMemory {
    /// Creates a new SafeMemory
    pub fn new(memory: Memory, n32: usize, prime: BigInt) -> Self {
        // TODO: Figure out a better way to calculate these
        let short_max = BigInt::from(0x8000_0000u64);
        let short_min = BigInt::from_biguint(
            num_bigint::Sign::NoSign,
            BigUint::from_str(
                "21888242871839275222246405745257275088548364400416034343698204186575808495617",
            )
            .unwrap(),
        ) - &short_max;
        let r_inv = BigInt::from_str(
            "9915499612839321149637521777990102151350674507940716049588462388200839649614",
        )
        .unwrap();

        Self {
            memory,
            prime,

            short_max,
            short_min,
            r_inv,
            n32,
        }
    }

    /// Gets an immutable view to the memory in 32 byte chunks
    pub fn view(&self) -> MemoryView<u32> {
        self.memory.view()
    }

    /// Returns the next free position in the memory
    pub fn free_pos(&self) -> u32 {
        self.view()[0].get()
    }

    /// Sets the next free position in the memory
    pub fn set_free_pos(&mut self, ptr: u32) {
        self.write_u32(0, ptr);
    }

    /// Allocates a U32 in memory
    pub fn alloc_u32(&mut self) -> u32 {
        let p = self.free_pos();
        self.set_free_pos(p + 8);
        p
    }

    /// Writes a u32 to the specified memory offset
    pub fn write_u32(&mut self, ptr: usize, num: u32) {
        let buf = unsafe { self.memory.data_unchecked_mut() };
        buf[ptr..ptr + std::mem::size_of::<u32>()].copy_from_slice(&num.to_le_bytes());
    }

    /// Reads a u32 from the specified memory offset
    pub fn read_u32(&self, ptr: usize) -> u32 {
        let buf = unsafe { self.memory.data_unchecked() };

        let mut bytes = [0; 4];
        bytes.copy_from_slice(&buf[ptr..ptr + std::mem::size_of::<u32>()]);

        u32::from_le_bytes(bytes)
    }

    /// Allocates `self.n32 * 4 + 8` bytes in the memory
    pub fn alloc_fr(&mut self) -> u32 {
        let p = self.free_pos();
        self.set_free_pos(p + self.n32 as u32 * 4 + 8);
        p
    }

    /// Writes a Field Element to memory at the specified offset, truncating
    /// to smaller u32 types if needed and adjusting the sign via 2s complement
    pub fn write_fr<E: ScalarEngine>(&mut self, ptr: usize, fr: &BigInt) -> Result<()> {
        if fr < &self.short_max && fr > &self.short_min {
            if fr >= &BigInt::zero() {
                self.write_short_positive(ptr, fr)?;
            } else {
                self.write_short_negative(ptr, fr)?;
            }
        } else {
            self.write_long_normal::<E>(ptr, fr)?;
        }

        Ok(())
    }

    /// Reads a Field Element from the memory at the specified offset
    pub fn read_fr<E: ScalarEngine>(&self, ptr: usize) -> Result<BigInt> {
        let view = self.memory.view::<u8>();

        let res = if view[ptr + 4 + 3].get() & 0x80 != 0 {
            let mut num = self.read_big::<E>(ptr + 8, self.n32)?;
            if view[ptr + 4 + 3].get() & 0x40 != 0 {
                num = (num * &self.r_inv) % &self.prime
            }
            num
        } else if view[ptr + 3].get() & 0x40 != 0 {
            let mut num = self.read_u32(ptr).into();
            // handle small negative
            num -= BigInt::from(0x100000000i64);
            num
        } else {
            self.read_u32(ptr).into()
        };

        Ok(res)
    }

    fn write_short_positive(&mut self, ptr: usize, fr: &BigInt) -> Result<()> {
        let num = fr.to_i32().expect("not a short positive");
        self.write_u32(ptr, num as u32);
        self.write_u32(ptr + 4, 0);
        Ok(())
    }

    fn write_short_negative(&mut self, ptr: usize, fr: &BigInt) -> Result<()> {
        // 2s complement
        let num = fr - &self.short_min;
        let num = num - &self.short_max;
        let num = num + BigInt::from(0x0001_0000_0000i64);

        let num = num
            .to_u32()
            .expect("could not cast as u32 (should never happen)");

        self.write_u32(ptr, num);
        self.write_u32(ptr + 4, 0);
        Ok(())
    }

    fn write_long_normal<E: ScalarEngine>(&mut self, ptr: usize, fr: &BigInt) -> Result<()> {
        self.write_u32(ptr, 0);
        self.write_u32(ptr + 4, i32::MIN as u32); // 0x80000000
        self.write_big::<E>(ptr + 8, fr)?;
        Ok(())
    }

    fn write_big<E: ScalarEngine>(&self, ptr: usize, num: &BigInt) -> Result<()> {
        let buf = unsafe { self.memory.data_unchecked_mut() };
        let num = E::Fr::from_str(&num.to_string()).unwrap();

        let repr = num.into_repr();
        let required_length = repr.as_ref().len() * 8;
        let mut bytes: Vec<u8> = Vec::with_capacity(required_length);
        repr.write_le(&mut bytes).unwrap();
        let len = bytes.len();
        buf[ptr..ptr + len].copy_from_slice(&bytes);

        Ok(())
    }

    /// Reads `num_bytes * 32` from the specified memory offset in a Big Integer
    pub fn read_big<E: ScalarEngine>(&self, ptr: usize, num_bytes: usize) -> Result<BigInt> {
        let buf = unsafe { self.memory.data_unchecked() };
        let mut buf = &buf[ptr..ptr + num_bytes * 32];

        let mut repr = E::Fr::zero().into_repr();
        repr.read_le(&mut buf)?;
        let fr = E::Fr::from_repr(repr)?;

        let big = BigUint::from_str_radix(&to_hex(&fr), 16)?;
        Ok(big.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::ToPrimitive;
    use std::str::FromStr;
    use wasmer::{MemoryType, Store};

    fn new() -> SafeMemory {
        SafeMemory::new(
            Memory::new(&Store::default(), MemoryType::new(1, None, false)).unwrap(),
            2,
            BigInt::from_str(
                "21888242871839275222246405745257275088548364400416034343698204186575808495617",
            )
            .unwrap(),
        )
    }

    #[test]
    fn i32_bounds() {
        let mem = new();
        let i32_max = i32::MAX as i64 + 1;
        assert_eq!(mem.short_min.to_i64().unwrap(), -i32_max);
        assert_eq!(mem.short_max.to_i64().unwrap(), i32_max);
    }

    #[test]
    fn read_write_32() {
        let mut mem = new();
        let num = u32::MAX;

        let inp = mem.read_u32(0);
        assert_eq!(inp, 0);

        mem.write_u32(0, num);
        let inp = mem.read_u32(0);
        assert_eq!(inp, num);
    }

    #[test]
    fn read_write_fr_small_positive() {
        read_write_fr(BigInt::from(1_000_000));
    }

    #[test]
    fn read_write_fr_small_negative() {
        read_write_fr(BigInt::from(-1_000_000));
    }

    #[test]
    fn read_write_fr_big_positive() {
        read_write_fr(BigInt::from(500000000000i64));
    }

    // TODO: How should this be handled?
    #[test]
    #[ignore]
    fn read_write_fr_big_negative() {
        read_write_fr(BigInt::from_str("-500000000000").unwrap())
    }

    fn read_write_fr(num: BigInt) {
        use crate::bellman_ce::pairing::bn256::Bn256;
        let mut mem = new();
        mem.write_fr::<Bn256>(0, &num).unwrap();
        let res = mem.read_fr::<Bn256>(0).unwrap();
        assert_eq!(res, num);
    }
}
