// FIXME: DON't use this library for scalar operation
#![cfg(not(feature = "wasm"))]
#![allow(unused_imports)]
use crate::bellman_ce::ScalarEngine;
use crate::ff::*;

#[derive(Debug, Eq)]
pub struct Fr(pub FrRepr);
/// This is the modulus m of the prime field
const MODULUS: FrRepr = FrRepr([18446744069414584321u64]);
/// The number of bits needed to represent the modulus.
const MODULUS_BITS: u32 = 64u32;
/// The number of bits that must be shaved from the beginning of
/// the representation when randomly sampling.
//const REPR_SHAVE_BITS: u32 = 0u32;
/// Precalculated mask to shave bits from the top limb in random sampling
// const TOP_LIMB_SHAVE_MASK: u64 = 0u64;
const TOP_LIMB_SHAVE_MASK: u64 = 0x7FFFFFFFFFFFFFFF;
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
const S: u32 = 31u32;
/// 2^s root of unity computed by GENERATOR^t
const ROOT_OF_UNITY: FrRepr = FrRepr([959634606461954525u64]);

#[derive(Debug, Eq)]
pub struct FrRepr(pub [u64; 1usize]);
#[automatically_derived]
impl std::marker::Copy for FrRepr {}
#[automatically_derived]
impl std::clone::Clone for FrRepr {
    #[inline]
    fn clone(&self) -> FrRepr {
        *self
    }
}

impl std::cmp::PartialEq for FrRepr {
    #[inline]
    fn eq(&self, other: &FrRepr) -> bool {
        self.0 == other.0
    }
}

#[automatically_derived]
impl std::default::Default for FrRepr {
    #[inline]
    fn default() -> FrRepr {
        FrRepr(std::default::Default::default())
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
        write!(f, "0x")?;
        for i in self.0.iter().rev() {
            write!(f, "{:016x}", *i)?;
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
    fn cmp(&self, other: &FrRepr) -> ::std::cmp::Ordering {
        for (a, b) in self.0.iter().rev().zip(other.0.iter().rev()) {
            if a < b {
                return ::std::cmp::Ordering::Less;
            } else if a > b {
                return ::std::cmp::Ordering::Greater;
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
        if n as usize >= 64 * 1usize {
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
    }
    #[inline(always)]
    fn shl(&mut self, mut n: u32) {
        if n as usize >= 64 * 1usize {
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
        let mut ret = (1usize as u32) * 64;
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
/// Elements are ordered lexicographically.
impl Ord for Fr {
    #[inline(always)]
    fn cmp(&self, other: &Fr) -> ::std::cmp::Ordering {
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
        write!(f, "0x{}", self.0)
    }
}
impl ::rand::Rand for Fr {
    /// Computes a uniformly random element using rejection sampling.
    fn rand<R: ::rand::Rng>(rng: &mut R) -> Self {
        loop {
            let mut tmp = Fr(FrRepr::rand(rng));
            tmp.0.as_mut()[0usize] &= TOP_LIMB_SHAVE_MASK;
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
                format!("{}", r.0)
            }))
        }
    }
    fn from_raw_repr(r: FrRepr) -> Result<Self, crate::ff::PrimeFieldDecodingError> {
        let r = Fr(r);
        if r.is_valid() {
            Ok(r)
        } else {
            Err(crate::ff::PrimeFieldDecodingError::NotInField({
                format!("{}", r.0)
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
            self.0.add_nocarry(&MODULUS);
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
    fn inverse(&self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            let one = FrRepr::from(1);
            let mut u = self.0;
            let mut v = MODULUS;
            let mut b = Fr(R2);
            let mut c = Self::zero();
            while u != one && v != one {
                while u.is_even() {
                    u.div2();
                    if b.0.is_even() {
                        b.0.div2();
                    } else {
                        b.0.add_nocarry(&MODULUS);
                        b.0.div2();
                    }
                }
                while v.is_even() {
                    v.div2();
                    if c.0.is_even() {
                        c.0.div2();
                    } else {
                        c.0.add_nocarry(&MODULUS);
                        c.0.div2();
                    }
                }
                if v < u {
                    u.sub_noborrow(&v);
                    b.sub_assign(&c);
                } else {
                    v.sub_noborrow(&u);
                    c.sub_assign(&b);
                }
            }
            if u == one {
                Some(b)
            } else {
                Some(c)
            }
        }
    }
    #[inline(always)]
    fn frobenius_map(&mut self, _: usize) {}
    #[inline]
    fn mul_assign(&mut self, other: &Fr) {
        let mut carry = 0;
        let r0 = crate::ff::mac_with_carry(0, (self.0).0[0usize], (other.0).0[0usize], &mut carry);
        let r1 = carry;
        self.mont_reduce(r0, r1);
    }
    #[inline]
    fn square(&mut self) {
        let r1 = 0;
        let mut carry = 0;
        let r0 = crate::ff::mac_with_carry(0, (self.0).0[0usize], (self.0).0[0usize], &mut carry);
        let r1 = crate::ff::adc(r1, 0, &mut carry);
        self.mont_reduce(r0, r1);
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
        r1 = crate::ff::adc(r1, 0, &mut carry);
        (self.0).0[0usize] = r1;
        self.reduce();
    }
}
impl crate::ff::SqrtField for Fr {
    fn legendre(&self) -> crate::ff::LegendreSymbol {
        let s = self.pow([4611686017353646080u64]);
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

#[derive(Clone, Copy, Debug)]
pub struct GL;
impl ScalarEngine for GL {
    type Fr = Fr;
}
