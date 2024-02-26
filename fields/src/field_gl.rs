#![allow(unused_imports)]

use crate::ff::*;
use core::ops::{Add, Div, Mul, Neg, Sub};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Eq, Serialize, Deserialize)]
pub struct Fr(pub FrRepr);

/// This is the modulus m of the prime field
pub const MODULUS: FrRepr = FrRepr([18446744069414584321u64]);
/// The number of bits needed to represent the modulus.
const MODULUS_BITS: u32 = 64u32;
/// 2^{limbs*64} mod m
const R: FrRepr = FrRepr([18446744065119617025u64]);
/// 2^{limbs*64*2} mod m
const R2: FrRepr = FrRepr([4294967295u64]);
/// -(m^{-1} mod m) mod m
const INV: u64 = 18446744069414584319u64;
/// Multiplicative generator of `MODULUS` - 1 order, also quadratic
/// nonresidue.
const GENERATOR: FrRepr = FrRepr([18446744039349813249u64]);
/// 2^s * t = MODULUS - 1 with t odd
const S: u32 = 32u32;
/// 2^s root of unity computed by GENERATOR^t
pub const ROOT_OF_UNITY: FrRepr = FrRepr([959634606461954525u64]);

#[derive(Eq, Serialize, Deserialize)]
pub struct FrRepr(pub [u64; 1usize]);

#[automatically_derived]
impl ::core::marker::Copy for FrRepr {}

#[automatically_derived]
impl std::clone::Clone for FrRepr {
    #[inline]
    fn clone(&self) -> FrRepr {
        //let _: std::clone::AssertParamIsClone<[u64; 2usize]>;
        *self
    }
}

#[automatically_derived]
impl ::core::cmp::PartialEq for FrRepr {
    #[inline]
    fn eq(&self, other: &FrRepr) -> bool {
        self.0 == other.0
    }
}
/*
#[automatically_derived]
impl std::marker::StructuralEq for FrRepr {}
#[automatically_derived]
impl std::cmp::Eq for FrRepr {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: std::cmp::AssertParamIsEq<[u64; 2usize]>;
    }
}
*/
#[automatically_derived]
impl ::core::default::Default for FrRepr {
    #[inline]
    fn default() -> FrRepr {
        FrRepr(::core::default::Default::default())
    }
}

impl ::std::fmt::Debug for FrRepr {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.write_fmt(format_args!("0x"))?;
        for i in self.0.iter().rev() {
            f.write_fmt(format_args!("{0:016x}", *i))?;
        }
        Ok(())
    }
}

impl ::rand::Rand for FrRepr {
    #[inline(always)]
    fn rand<R: ::rand::Rng>(rng: &mut R) -> Self {
        FrRepr(rng.gen())
    }
}

impl ::std::fmt::Display for FrRepr {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.write_fmt(format_args!("0x"))?;
        for i in self.0.iter().rev() {
            f.write_fmt(format_args!("{0:016x}", *i))?;
        }
        Ok(())
    }
}

impl std::hash::Hash for FrRepr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for limb in self.0.iter() {
            limb.hash(state);
        }
    }
}

impl AsRef<[u64]> for FrRepr {
    #[inline(always)]
    fn as_ref(&self) -> &[u64] {
        &self.0
    }
}

impl AsMut<[u64]> for FrRepr {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u64] {
        &mut self.0
    }
}

impl From<u64> for FrRepr {
    #[inline(always)]
    fn from(val: u64) -> FrRepr {
        use std::default::Default;
        let mut repr = Self::default();
        repr.0[0] = val;
        repr
    }
}

impl Ord for FrRepr {
    #[inline(always)]
    fn cmp(&self, other: &FrRepr) -> Ordering {
        for (a, b) in self.0.iter().rev().zip(other.0.iter().rev()) {
            if *a != *b {
                return a.cmp(b);
            }
        }
        ::std::cmp::Ordering::Equal
    }
}

impl PartialOrd for FrRepr {
    #[inline(always)]
    fn partial_cmp(&self, other: &FrRepr) -> Option<::std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl crate::ff::PrimeFieldRepr for FrRepr {
    #[inline(always)]
    fn is_odd(&self) -> bool {
        self.0[0] & 1 == 1
    }
    #[inline(always)]
    fn is_even(&self) -> bool {
        !self.is_odd()
    }
    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.0.iter().all(|&e| e == 0)
    }
    #[inline(always)]
    fn shr(&mut self, mut n: u32) {
        if n as usize >= 64 * 2usize {
            *self = Self::from(0);
            return;
        }
        while n >= 64 {
            let mut t = 0;
            for i in self.0.iter_mut().rev() {
                ::std::mem::swap(&mut t, i);
            }
            n -= 64;
        }
        if n > 0 {
            let mut t = 0;
            for i in self.0.iter_mut().rev() {
                let t2 = *i << (64 - n);
                *i >>= n;
                *i |= t;
                t = t2;
            }
        }
    }
    #[inline(always)]
    fn div2(&mut self) {
        let mut t = 0;
        for i in self.0.iter_mut().rev() {
            let t2 = *i << 63;
            *i >>= 1;
            *i |= t;
            t = t2;
        }
    }
    #[inline(always)]
    fn mul2(&mut self) {
        let mut last = 0;
        for i in &mut self.0 {
            let tmp = *i >> 63;
            *i <<= 1;
            *i |= last;
            last = tmp;
        }
        if last > 0 {
            self.0[0] += R2.0[0];
        }
    }
    #[inline(always)]
    fn shl(&mut self, mut n: u32) {
        if n as usize >= 64 * 2usize {
            *self = Self::from(0);
            return;
        }
        while n >= 64 {
            let mut t = 0;
            for i in &mut self.0 {
                ::std::mem::swap(&mut t, i);
            }
            n -= 64;
        }
        if n > 0 {
            let mut t = 0;
            for i in &mut self.0 {
                let t2 = *i >> (64 - n);
                *i <<= n;
                *i |= t;
                t = t2;
            }
        }
    }
    #[inline(always)]
    fn num_bits(&self) -> u32 {
        let mut ret = (2usize as u32) * 64;
        for i in self.0.iter().rev() {
            let leading = i.leading_zeros();
            ret -= leading;
            if leading != 64 {
                break;
            }
        }
        ret
    }
    #[inline(always)]
    fn add_nocarry(&mut self, other: &FrRepr) {
        let mut carry = 0;
        for (a, b) in self.0.iter_mut().zip(other.0.iter()) {
            *a = crate::ff::adc(*a, *b, &mut carry);
        }
        if carry > 0 {
            // 2**64 - (2**64 - 2**32 + 1)
            self.0[0] += R2.0[0];
        }
    }
    #[inline(always)]
    fn sub_noborrow(&mut self, other: &FrRepr) {
        let mut borrow = 0;
        for (a, b) in self.0.iter_mut().zip(other.0.iter()) {
            *a = crate::ff::sbb(*a, *b, &mut borrow);
        }
    }
}

impl ::std::marker::Copy for Fr {}

impl ::std::clone::Clone for Fr {
    fn clone(&self) -> Fr {
        *self
    }
}

impl ::std::cmp::PartialEq for Fr {
    fn eq(&self, other: &Fr) -> bool {
        self.0 == other.0
    }
}

//impl ::std::cmp::Eq for Fr {}
impl ::std::fmt::Debug for Fr {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.write_fmt(format_args!("{0}({1:?})", "Fr", self.into_repr()))
    }
}

/// Elements are ordered lexicographically.
impl Ord for Fr {
    #[inline(always)]
    fn cmp(&self, other: &Fr) -> std::cmp::Ordering {
        self.into_repr().cmp(&other.into_repr())
    }
}

impl PartialOrd for Fr {
    #[inline(always)]
    fn partial_cmp(&self, other: &Fr) -> Option<::std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ::std::fmt::Display for Fr {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.write_fmt(format_args!("{0}({1})", "Fr", self.into_repr()))
    }
}

impl ::rand::Rand for Fr {
    /// Computes a uniformly random element using rejection sampling.
    fn rand<R: ::rand::Rng>(rng: &mut R) -> Self {
        loop {
            let tmp = Fr(FrRepr::rand(rng));
            if tmp.is_valid() {
                return tmp;
            }
        }
    }
}

impl From<Fr> for FrRepr {
    fn from(e: Fr) -> FrRepr {
        e.into_repr()
    }
}

impl crate::ff::PrimeField for Fr {
    type Repr = FrRepr;
    fn from_repr(r: FrRepr) -> Result<Fr, crate::ff::PrimeFieldDecodingError> {
        let mut r = Fr(r);
        if r.is_valid() {
            r.mul_assign(&Fr(R2));
            Ok(r)
        } else {
            Err(crate::ff::PrimeFieldDecodingError::NotInField({
                let res = std::fmt::format(format_args!("{0}", r.0));
                res
            }))
        }
    }
    fn from_raw_repr(r: FrRepr) -> Result<Self, crate::ff::PrimeFieldDecodingError> {
        let r = Fr(r);
        if r.is_valid() {
            Ok(r)
        } else {
            Err(crate::ff::PrimeFieldDecodingError::NotInField({
                let res = std::fmt::format(format_args!("{0}", r.0));
                res
            }))
        }
    }
    fn into_repr(&self) -> FrRepr {
        let mut r = *self;
        r.mont_reduce((self.0).0[0usize], 0);
        r.0
    }
    fn into_raw_repr(&self) -> FrRepr {
        let r = *self;
        r.0
    }
    fn char() -> FrRepr {
        MODULUS
    }
    const NUM_BITS: u32 = MODULUS_BITS;
    const CAPACITY: u32 = Self::NUM_BITS - 1;
    fn multiplicative_generator() -> Self {
        Fr(GENERATOR)
    }
    const S: u32 = S;
    fn root_of_unity() -> Self {
        Fr(ROOT_OF_UNITY)
    }
}

impl crate::ff::Field for Fr {
    #[inline]
    fn zero() -> Self {
        Fr(FrRepr::from(0))
    }
    #[inline]
    fn one() -> Self {
        Fr(R)
    }
    #[inline]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
    #[inline]
    fn add_assign(&mut self, other: &Fr) {
        self.0.add_nocarry(&other.0);
        self.reduce();
    }
    #[inline]
    fn double(&mut self) {
        self.0.mul2();
        self.reduce();
    }
    #[inline]
    fn sub_assign(&mut self, other: &Fr) {
        if other.0 > self.0 {
            let mut tmp = MODULUS;
            tmp.sub_noborrow(&other.0);
            self.0.add_nocarry(&tmp);
            return;
        }
        self.0.sub_noborrow(&other.0);
    }
    #[inline]
    fn negate(&mut self) {
        if !self.is_zero() {
            let mut tmp = MODULUS;
            tmp.sub_noborrow(&self.0);
            self.0 = tmp;
        }
    }

    /// borrow from https://github.com/facebook/winterfell/blob/main/math/src/field/f64/mod.rs#L142
    #[inline]
    fn inverse(&self) -> Option<Self> {
        // compute base^(M - 2) using 72 multiplications
        // M - 2 = 0b1111111111111111111111111111111011111111111111111111111111111111

        // compute base^11
        let mut sf = *self;
        sf.square();
        sf.mul_assign(self);

        // compute base^111
        sf.square();
        let t3 = sf * *self;

        // compute base^111111 (6 ones)
        let t6 = exp_acc::<3>(t3, t3);

        // compute base^111111111111 (12 ones)
        let t12 = exp_acc::<6>(t6, t6);

        // compute base^111111111111111111111111 (24 ones)
        let t24 = exp_acc::<12>(t12, t12);

        // compute base^1111111111111111111111111111111 (31 ones)
        let mut t30 = exp_acc::<6>(t24, t6);
        t30.square();
        let t31 = t30 * *self;

        // compute base^111111111111111111111111111111101111111111111111111111111111111
        let mut t63 = exp_acc::<32>(t31, t31);

        // compute base^1111111111111111111111111111111011111111111111111111111111111111

        t63.square();
        Some(t63 * *self)
    }

    #[inline(always)]
    fn frobenius_map(&mut self, _: usize) {}
    #[inline]
    fn mul_assign(&mut self, other: &Fr) {
        let mut carry = 0;
        let r0 = crate::ff::mac_with_carry(0, (self.0).0[0usize], (other.0).0[0usize], &mut carry);
        self.mont_reduce(r0, carry);
    }
    #[inline]
    fn square(&mut self) {
        self.mul_assign(&self.clone());
    }
}

impl Fr {
    #[inline]
    pub fn exp(self, power: u64) -> Fr {
        let mut b: Fr;
        let mut r = Fr::one();
        for i in (0..64).rev() {
            r.square();
            b = r;
            b = b * self;
            // Constant-time branching
            let mask = -(((power >> i) & 1 == 1) as i64) as u64;
            r.0 .0[0] ^= mask & (r.0 .0[0] ^ b.0 .0[0]);
        }
        r
    }
}

impl std::default::Default for Fr {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::hash::Hash for Fr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for limb in self.0.as_ref().iter() {
            limb.hash(state);
        }
    }
}

impl From<u64> for Fr {
    #[inline(always)]
    fn from(val: u64) -> Fr {
        Fr::from_repr(FrRepr::from(val)).unwrap()
    }
}

impl From<Fr> for u64 {
    fn from(val: Fr) -> Self {
        val.0 .0[0]
    }
}

impl Fr {
    /// Determines if the element is really in the field. This is only used
    /// internally.
    #[inline(always)]
    fn is_valid(&self) -> bool {
        self.0 < MODULUS
    }
    /// Subtracts the modulus from this element if this element is not in the
    /// field. Only used interally.
    #[inline(always)]
    fn reduce(&mut self) {
        if !self.is_valid() {
            self.0.sub_noborrow(&MODULUS);
        }
    }
    #[inline(always)]
    fn mont_reduce(&mut self, r0: u64, mut r1: u64) {
        let k = r0.wrapping_mul(INV);
        let mut carry = 0;
        crate::ff::mac_with_carry(r0, k, MODULUS.0[0], &mut carry);
        r1 = crate::ff::mac_with_carry(r1, k, 0, &mut carry);
        let mut r2 = crate::ff::adc(0, 0, &mut carry);
        let k = r1.wrapping_mul(INV);
        let mut carry = 0;
        crate::ff::mac_with_carry(r1, k, MODULUS.0[0], &mut carry);
        r2 = crate::ff::mac_with_carry(r2, k, 0, &mut carry);
        (self.0).0[0usize] = r2;
        // (self.0).0[1usize] = r3;
        self.reduce();
    }

    pub const ZERO: Self = Self(FrRepr([0]));
    pub const ONE: Self = Self(R);
    pub fn as_int(&self) -> u64 {
        self.into_repr().0[0]
    }
}

impl crate::ff::SqrtField for Fr {
    fn legendre(&self) -> crate::ff::LegendreSymbol {
        let s = self.pow([9223372034707292160u64]);
        if s == Self::zero() {
            crate::ff::LegendreSymbol::Zero
        } else if s == Self::one() {
            crate::ff::LegendreSymbol::QuadraticResidue
        } else {
            crate::ff::LegendreSymbol::QuadraticNonResidue
        }
    }
    fn sqrt(&self) -> Option<Self> {
        match self.legendre() {
            crate::ff::LegendreSymbol::Zero => Some(*self),
            crate::ff::LegendreSymbol::QuadraticNonResidue => None,
            crate::ff::LegendreSymbol::QuadraticResidue => {
                let mut c = Fr(ROOT_OF_UNITY);
                let mut r = self.pow([2147483648u64]);
                let mut t = self.pow([4294967295u64]);
                let mut m = S;
                while t != Self::one() {
                    let mut i = 1;
                    {
                        let mut t2i = t;
                        t2i.square();
                        loop {
                            if t2i == Self::one() {
                                break;
                            }
                            t2i.square();
                            i += 1;
                        }
                    }
                    for _ in 0..(m - i - 1) {
                        c.square();
                    }
                    r.mul_assign(&c);
                    c.square();
                    t.mul_assign(&c);
                    m = i;
                }
                Some(r)
            }
        }
    }
}

impl Add for Fr {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        let mut lhs = self;
        lhs.add_assign(&other);
        lhs
    }
}

impl Mul for Fr {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
        let mut lhs = self;
        lhs.mul_assign(&other);
        lhs
    }
}

impl Sub for Fr {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        let mut lhs = self;
        lhs.sub_assign(&other);
        lhs
    }
}

impl Div for Fr {
    type Output = Self;

    #[inline]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self {
        self * rhs.inverse().unwrap()
    }
}

impl Neg for Fr {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        let mut tmp = self;
        tmp.negate();
        tmp
    }
}

/// Squares the base N number of times and multiplies the result by the tail value.
#[inline(always)]
fn exp_acc<const N: usize>(base: Fr, tail: Fr) -> Fr {
    let mut result = base;
    for _ in 0..N {
        result.square();
    }
    result * tail
}

#[derive(Clone, Copy, Debug)]
pub struct GL;

impl ScalarEngine for GL {
    type Fr = Fr;
}

#[cfg(test)]
mod tests {
    use super::Field;
    use super::Fr;
    use super::FrRepr;
    use super::PrimeField;
    use crate::ff::*;
    use crate::rand::Rand;
    use proptest::prelude::*;
    use std::ops::{Add, Div, Mul, Neg, Sub};

    proptest! {
        #[test]
        fn gl_check_add(a in any::<u64>()) {
            let v = Fr::from_str(&a.to_string()).unwrap();
            let added = v + v;
            let double = v * Fr::from_str("2").unwrap();
            prop_assert_eq!(added, double);
        }

        #[test]
        fn gl_check_sub(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Fr::from_str(&a.to_string()).unwrap();
            let v2 = Fr::from_str(&b.to_string()).unwrap();
            let lhs = v2 - v1;
            let rhs = lhs + v1;
            prop_assert_eq!(v2, rhs);
        }

        #[test]
        fn gl_check_mul(a in any::<u64>()) {
            let v = Fr::from_str(&a.to_string()).unwrap();
            let lhs = v * v * v;
            let mut rhs = v;
            rhs.square();
            prop_assert_eq!(lhs, rhs * v);
        }

        #[test]
        fn gl_check_inv(a in any::<u64>()) {
            let v = Fr::from_str(&a.to_string()).unwrap();
            let v_inversed = v.inverse().unwrap();
            prop_assert_eq!(v * v_inversed, Fr::one());
        }

        #[test]
        fn gl_check_div(a in any::<u64>(), b in any::<u64>()){
            let v1 = Fr::from_str(&a.to_string()).unwrap();
            let v2 = Fr::from_str(&b.to_string()).unwrap();
            let result = v1 / v2;
            prop_assert_eq!(result * v2, v1);
        }

        #[test]
        fn gl_check_neg(a in any::<u64>()){
            let mut v1 = Fr::from_str(&a.to_string()).unwrap();
            let v2 = v1;
            v1.negate();
            prop_assert_eq!(v1+v2, Fr::zero());
        }

        #[test]
        fn gl_check_sqrt(a in any::<u64>()) {
            let v = Fr::from_str(&a.to_string()).unwrap();
            match v.sqrt() {
                Some(mut sq_v) => {
                    sq_v.square();
                    prop_assert_eq!(v, sq_v);
                }
                _ => {
                    prop_assert_eq!(v.legendre(), crate::ff::LegendreSymbol::QuadraticNonResidue);
                }
            }
        }
    }

    #[test]
    fn test_serde_and_deserde() {
        let data = Fr::one();
        let serialized = serde_json::to_string(&data).unwrap();
        println!("Serialized: {}", serialized);

        let expect: Fr = serde_json::from_str(&serialized).unwrap();

        assert_eq!(data, expect);
    }
}
