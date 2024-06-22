#![allow(dead_code)]
use crate::traits::FieldExtension;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use ff::derive::subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use ff::{Field, PrimeField};
use fields::field_gl::Goldilocks as Fr;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::slice;

use core::fmt::{Display, Formatter};
/// GF(2^3) implementation
/// Prime: 0xFFFFFFFF00000001
/// Irreducible polynomial: x^3 - x -1
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct F3G {
    pub cube: [Fr; 3],
    pub dim: usize,
}

unsafe impl Send for F3G {}
unsafe impl Sync for F3G {}

impl Hash for F3G {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
        self.dim.hash(state);
    }
}

/// Field modulus = 2^64 - 2^32 + 1
const M: u64 = 0xFFFFFFFF00000001;

/// 2^128 mod M; this is used for conversion of elements into Montgomery representation.
const R2: u64 = 0xFFFFFFFE00000001;

/// 2^32 root of unity
const G: u64 = 1753635133440165772;

/// Number of bytes needed to represent field element
const ELEMENT_BYTES: usize = core::mem::size_of::<u64>();

impl F3G {
    pub fn new(a: Fr, b: Fr, c: Fr) -> Self {
        F3G {
            cube: [a, b, c],
            dim: 3,
        }
    }
}
impl FieldExtension for F3G {
    const ELEMENT_BYTES: usize = ELEMENT_BYTES;
    const ZEROS: Self = Self {
        cube: [Fr::ZERO, Fr::ZERO, Fr::ZERO],
        dim: 3,
    };
    const ONES: Self = Self {
        cube: [Fr::ONE, Fr::ONE, Fr::ONE],
        dim: 3,
    };

    fn as_elements(&self) -> Vec<Fr> {
        self.cube.to_vec()
    }

    #[inline(always)]
    fn to_be(&self) -> Fr {
        assert_eq!(self.dim, 1);
        self.as_elements()[0]
    }

    #[inline]
    fn _eq(&self, rhs: &Self) -> bool {
        self.ct_eq(rhs).into()
    }

    #[inline]
    fn gt(&self, rhs: &Self) -> bool {
        assert_eq!(self.dim, rhs.dim); // FIXME: align with JS
        let les = self.as_elements();
        let res = rhs.as_elements();
        match self.dim {
            3 => {
                (les[0] > res[0])
                    || ((les[0] == res[0]) && (les[1] > res[1]))
                    || ((les[0] == res[0]) && (les[1] == res[1]) && (les[2] > res[2]))
            }
            1 => les[0] > res[0],
            _ => {
                panic!("Invalid dim");
            }
        }
    }

    #[inline]
    fn geq(&self, rhs: &Self) -> bool {
        self._eq(rhs) || self.gt(rhs)
    }

    #[inline]
    fn lt(&self, rhs: &Self) -> bool {
        !self.geq(rhs)
    }

    #[inline]
    fn exp(&self, e_: usize) -> Self {
        let mut e = e_;
        if e == 0 {
            return Self::ONE;
        }
        let mut bits = Vec::<i32>::new();

        while e != 0 {
            if (e & 1) == 1 {
                bits.push(1);
            } else {
                bits.push(0);
            }
            e >>= 1;
        }

        if bits.is_empty() {
            return Self::ONE;
        }

        let mut res = *self;
        for i in (0..bits.len() - 1).rev() {
            res = res.square();
            if bits[i] == 1 {
                res = res.mul(*self);
            }
        }
        res
    }

    #[inline]
    fn inv(&self) -> Self {
        println!("test: {:?}", self.dim);
        match self.dim {
            3 => self._inv(),
            1 => Self::from(self.to_be().invert().unwrap()),
            _ => {
                panic!("Invalid dim");
            }
        }
    }

    #[inline(always)]
    fn dim(&self) -> usize {
        self.dim
    }

    #[inline(always)]
    fn from_vec(values: Vec<Fr>) -> Self {
        assert_eq!(values.len(), 3);
        Self {
            cube: [values[0], values[1], values[2]],
            dim: 3,
        }
    }

    #[inline]
    fn mul_scalar(&self, b: usize) -> Self {
        let b = Fr::from(b as u64);
        let elems = self.as_elements();
        if self.dim == 1 {
            Self::from(elems[0] * b)
        } else {
            Self::new(elems[0] * b, elems[1] * b, elems[2] * b)
        }
    }

    #[inline]
    fn leq(&self, rhs: &Self) -> bool {
        !self.gt(rhs)
    }

    #[inline]
    fn as_int(&self) -> u64 {
        /*
        if self.dim == 1 {
            self.to_be().as_int()
        } else {
            panic!("Invalid as int: {:?}", *self);
        }
        */
        Fr::render_repr_to_str(self.as_elements()[0].to_repr())
    }

    #[inline]
    fn elements_as_bytes(elements: &[Self]) -> &[u8] {
        // TODO: take endianness into account.
        // let p = elements.as_ptr();
        // let len = elements.len() * Self::ELEMENT_BYTES;
        // unsafe { slice::from_raw_parts(p as *const u8, len) }
        todo!()
    }

    fn as_bytes(&self) -> &[u8] {
        // let self_ptr: *const Self = self;
        // unsafe { slice::from_raw_parts(self_ptr as *const u8, Self::ELEMENT_BYTES * self.dim) }
        todo!()
    }

    const IS_CANONICAL: bool = false;

    const NEW_SIZE: u64 = 0;
}

impl F3G {
    fn _inv(&self) -> Self {
        assert_eq!(self.dim, 3);
        let a = self.cube;
        let aa = a[0] * a[0];
        let ac = a[0] * a[2];
        let ba = a[1] * a[0];
        let bb = a[1] * a[1];
        let bc = a[1] * a[2];
        let cc = a[2] * a[2];

        let aaa = aa * a[0];
        let aac = aa * a[2];
        let abc = ba * a[2];
        let abb = ba * a[1];
        let acc = ac * a[2];
        let bbb = bb * a[1];
        let bcc = bc * a[2];
        let ccc = cc * a[2];

        let t = -aaa - aac - aac + abc + abc + abc + abb - acc - bbb + bcc - ccc;
        let tinv = t.invert().unwrap();

        let i1 = (-aa - ac - ac + bc + bb - cc) * tinv;
        let i2 = (ba - cc) * tinv;
        let i3 = (-bb + ac + cc) * tinv;

        Self {
            cube: [i1, i2, i3],
            dim: 3,
        }
    }
}

impl ConditionallySelectable for F3G {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut result = Self::default();
        for i in 0..3 {
            result.cube[i] = Fr::conditional_select(&a.cube[i], &b.cube[i], choice);
        }
        result.dim = if choice.unwrap_u8() == 1 {
            b.dim
        } else {
            a.dim
        };
        result
    }
}

impl std::iter::Sum for F3G {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::ZERO, |acc, item| acc + item)
    }
}

impl std::iter::Product for F3G {
    fn product<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::ONE, |acc, item| acc * item)
    }
}

impl<'a> std::iter::Sum<&'a Self> for F3G {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        iter.fold(Self::ZERO, |acc, &item| acc + item)
    }
}

impl<'a> std::iter::Product<&'a Self> for F3G {
    fn product<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        iter.fold(Self::ONE, |acc, &item| acc * item)
    }
}

impl ConstantTimeEq for F3G {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.cube[0].ct_eq(&other.cube[0])
            & self.cube[1].ct_eq(&other.cube[1])
            & self.cube[2].ct_eq(&other.cube[2])
    }
}

impl Neg for F3G {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            cube: [self.cube[0].neg(), self.cube[1].neg(), self.cube[2].neg()],
            dim: self.dim,
        }
    }
}

impl Add for F3G {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        match self.dim {
            3 => {
                if rhs.dim == 3 {
                    Self {
                        cube: [
                            self.cube[0] + rhs.cube[0],
                            self.cube[1] + rhs.cube[1],
                            self.cube[2] + rhs.cube[2],
                        ],
                        dim: 3,
                    }
                } else {
                    let r = self.as_elements();
                    Self::new(r[0] + rhs.to_be(), r[1], r[2])
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() + rhs.to_be())
                } else {
                    let r = rhs.as_elements();
                    Self::new(r[0] + self.to_be(), r[1], r[2])
                }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
    }
}

impl<'a> Add<&'a Self> for F3G {
    type Output = Self;

    #[inline]
    fn add(self, rhs: &'a Self) -> Self::Output {
        match self.dim {
            3 => {
                if rhs.dim == 3 {
                    Self {
                        cube: [
                            self.cube[0] + rhs.cube[0],
                            self.cube[1] + rhs.cube[1],
                            self.cube[2] + rhs.cube[2],
                        ],
                        dim: 3,
                    }
                } else {
                    let r = self.as_elements();
                    Self::new(r[0] + rhs.to_be(), r[1], r[2])
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() + rhs.to_be())
                } else {
                    let r = rhs.as_elements();
                    Self::new(r[0] + self.to_be(), r[1], r[2])
                }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
    }
}

impl AddAssign for F3G {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<'a> AddAssign<&'a Self> for F3G {
    #[inline]
    fn add_assign(&mut self, rhs: &'a Self) {
        *self = *self + rhs;
    }
}

impl Sub for F3G {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        match self.dim {
            3 => {
                if rhs.dim == 3 {
                    Self {
                        cube: [
                            self.cube[0] - rhs.cube[0],
                            self.cube[1] - rhs.cube[1],
                            self.cube[2] - rhs.cube[2],
                        ],
                        dim: 3,
                    }
                } else {
                    let r = self.as_elements();
                    Self::new(r[0] - rhs.to_be(), r[1], r[2])
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() - rhs.to_be())
                } else {
                    let r = rhs.as_elements();
                    Self::new(self.to_be() - r[0], -r[1], -r[2])
                }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
    }
}

impl<'a> Sub<&'a Self> for F3G {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: &'a Self) -> Self::Output {
        match self.dim {
            3 => {
                if rhs.dim == 3 {
                    Self {
                        cube: [
                            self.cube[0] - rhs.cube[0],
                            self.cube[1] - rhs.cube[1],
                            self.cube[2] - rhs.cube[2],
                        ],
                        dim: 3,
                    }
                } else {
                    let r = self.as_elements();
                    Self::new(r[0] - rhs.to_be(), r[1], r[2])
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() - rhs.to_be())
                } else {
                    let r = rhs.as_elements();
                    Self::new(self.to_be() - r[0], -r[1], -r[2])
                }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
    }
}

impl SubAssign for F3G {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<'a> SubAssign<&'a Self> for F3G {
    #[inline]
    fn sub_assign(&mut self, rhs: &'a Self) {
        *self = *self - rhs;
    }
}

impl Mul for F3G {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        match self.dim {
            3 => {
                // 3 * 1
                if rhs.dim == 1 {
                    let lhs = rhs.to_be();
                    let r = self.as_elements();
                    Self::new(lhs * r[0], lhs * r[1], lhs * r[2])
                } else {
                    let a = self.cube;
                    let b = rhs.cube;
                    let aa = (a[0] + a[1]) * (b[0] + b[1]);
                    let bb = (a[0] + a[2]) * (b[0] + b[2]);
                    let cc = (a[1] + a[2]) * (b[1] + b[2]);
                    let dd = a[0] * b[0];
                    let ee = a[1] * b[1];
                    let ff = a[2] * b[2];
                    let gg = dd - ee;

                    Self {
                        cube: [(cc + gg - ff), (aa + cc - ee - ee - dd), (bb - gg)],
                        dim: 3,
                    }
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() * rhs.to_be())
                } else {
                    // 1 * 3
                    let lhs = self.to_be();
                    let r = rhs.as_elements();
                    Self::new(lhs * r[0], lhs * r[1], lhs * r[2])
                }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
    }
}

impl<'a> Mul<&'a Self> for F3G {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: &'a Self) -> Self::Output {
        match self.dim {
            3 => {
                // 3 * 1
                if rhs.dim == 1 {
                    let lhs = rhs.to_be();
                    let r = self.as_elements();
                    Self::new(lhs * r[0], lhs * r[1], lhs * r[2])
                } else {
                    let a = self.cube;
                    let b = rhs.cube;
                    let aa = (a[0] + a[1]) * (b[0] + b[1]);
                    let bb = (a[0] + a[2]) * (b[0] + b[2]);
                    let cc = (a[1] + a[2]) * (b[1] + b[2]);
                    let dd = a[0] * b[0];
                    let ee = a[1] * b[1];
                    let ff = a[2] * b[2];
                    let gg = dd - ee;

                    Self {
                        cube: [(cc + gg - ff), (aa + cc - ee - ee - dd), (bb - gg)],
                        dim: 3,
                    }
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() * rhs.to_be())
                } else {
                    // 1 * 3
                    let lhs = self.to_be();
                    let r = rhs.as_elements();
                    Self::new(lhs * r[0], lhs * r[1], lhs * r[2])
                }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
    }
}

impl MulAssign for F3G {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<'a> MulAssign<&'a Self> for F3G {
    #[inline]
    fn mul_assign(&mut self, rhs: &'a Self) {
        *self = *self * rhs;
    }
}

impl ff::Field for F3G {
    const ZERO: Self = Self {
        cube: [Fr::ZERO, Fr::ZERO, Fr::ZERO],
        dim: 1,
    };

    const ONE: Self = Self {
        cube: [Fr::ONE, Fr::ZERO, Fr::ZERO],
        dim: 1,
    };

    fn random(mut rng: impl rand::RngCore) -> Self {
        let a = Fr::random(&mut rng);
        Self {
            cube: [a, Fr::ZERO, Fr::ZERO],
            dim: 1,
        }
    }

    fn square(&self) -> Self {
        match self.dim {
            3 => {
                let a = self.cube;
                let aa = (a[0] + a[1]) * (a[0] + a[1]);
                let bb = (a[0] + a[2]) * (a[0] + a[2]);
                let cc = (a[1] + a[2]) * (a[1] + a[2]);
                let dd = a[0] * a[0];
                let ee = a[1] * a[1];
                let ff = a[2] * a[2];
                let gg = dd - ee;

                Self {
                    cube: [(cc + gg - ff), (aa + cc - ee - ee - dd), (bb - gg)],
                    dim: 3,
                }
            }
            1 => Self::from(self.to_be() * self.to_be()),
            _ => {
                panic!("Invalid dim");
            }
        }
    }

    fn double(&self) -> Self {
        let mut out = *self;
        out += self;
        out
    }

    fn invert(&self) -> ff::derive::subtle::CtOption<Self> {
        todo!()
    }

    fn sqrt_ratio(num: &Self, div: &Self) -> (ff::derive::subtle::Choice, Self) {
        todo!()
    }

    fn is_zero(&self) -> ff::derive::subtle::Choice {
        match self.dim {
            1 => self.ct_eq(&Self::ZERO),
            _ => self.ct_eq(&Self::ZEROS),
        }
    }

    fn is_zero_vartime(&self) -> bool {
        self.is_zero().into()
    }

    fn cube(&self) -> Self {
        self.square() * self
    }

    fn sqrt_alt(&self) -> (ff::derive::subtle::Choice, Self) {
        Self::sqrt_ratio(self, &Self::ONE)
    }

    fn sqrt(&self) -> ff::derive::subtle::CtOption<Self> {
        let (is_square, res) = Self::sqrt_ratio(self, &Self::ONE);
        ff::derive::subtle::CtOption::new(res, is_square)
    }

    fn pow<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        let mut res = Self::ONE;
        for e in exp.as_ref().iter().rev() {
            for i in (0..64).rev() {
                res = res.square();
                let mut tmp = res;
                tmp *= self;
                res.conditional_assign(&tmp, (((*e >> i) & 1) as u8).into());
            }
        }
        res
    }

    fn pow_vartime<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        let mut res = Self::ONE;
        for e in exp.as_ref().iter().rev() {
            for i in (0..64).rev() {
                res = res.square();

                if ((*e >> i) & 1) == 1 {
                    res.mul_assign(self);
                }
            }
        }

        res
    }
}

impl From<Fr> for F3G {
    #[inline]
    fn from(value: Fr) -> Self {
        F3G {
            cube: [value, Fr::ZERO, Fr::ZERO],
            dim: 1,
        }
    }
}

impl From<u64> for F3G {
    #[inline]
    fn from(value: u64) -> Self {
        Self::from(Fr::from(value))
    }
}

impl From<i32> for F3G {
    #[inline]
    fn from(value: i32) -> Self {
        Self::from(Fr::from(value as u64))
    }
}

impl From<u32> for F3G {
    #[inline]
    fn from(value: u32) -> Self {
        Self::from(Fr::from(value as u64))
    }
}

impl From<usize> for F3G {
    #[inline]
    fn from(value: usize) -> Self {
        Self::from(Fr::from(value as u64))
    }
}

impl From<&i32> for F3G {
    #[inline]
    fn from(value: &i32) -> Self {
        Self::from(Fr::from(*value as u64))
    }
}

impl From<&usize> for F3G {
    #[inline]
    fn from(value: &usize) -> Self {
        Self::from(Fr::from(*value as u64))
    }
}

impl From<u16> for F3G {
    /// Converts a 16-bit value into a field element.
    #[inline]
    fn from(value: u16) -> Self {
        Self::from(value as u64)
    }
}

impl From<u8> for F3G {
    /// Converts an 8-bit value into a field element.
    #[inline]
    fn from(value: u8) -> Self {
        Self::from(value as u64)
    }
}

impl From<[u8; 8]> for F3G {
    /// Converts the value encoded in an array of 8 bytes into a field element. The bytes are
    /// assumed to encode the element in the canonical representation in little-endian byte order.
    /// If the value is greater than or equal to the field modulus, modular reduction is silently
    /// performed.
    #[inline]
    fn from(bytes: [u8; 8]) -> Self {
        let value = u64::from_le_bytes(bytes);
        Self::from(value)
    }
}

// // FIXME
// impl From<u128> for F3G {
//     /// Converts a 128-bit value into a field element.
//     fn from(_: u128) -> Self {
//         //const R3: u128 = 1 (= 2^192 mod M );// thus we get that mont_red_var((mont_red_var(x) as u128) * R3) becomes
//         //Self(mont_red_var(mont_red_var(x) as u128))  // Variable time implementation
//         //Self(mont_red_cst(mont_red_cst(x) as u128)) // Constant time implementation
//         panic!("Unimplement");
//     }
// }

impl Display for F3G {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let elems = self.as_elements();
        if self.dim == 1 {
            write!(f, "{:?}", elems[0])
        } else {
            write!(f, "[{:?},{:?},{:?}]", elems[0], elems[1], elems[2])
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::f3g::F3G;
    use crate::polutils::batch_inverse;
    use crate::traits::FieldExtension;
    use ff::Field;
    use fields::field_gl::Goldilocks as Fr;
    use rand::thread_rng;
    use std::ops::{Add, Mul};

    #[test]
    fn test_f3g_add() {
        let f1 = F3G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64));
        let f2 = f1.add(f1);
        assert_eq!(f2, f1.double());

        let f1 = F3G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64));
        let f2 = F3G::new(
            Fr::from(4u64),
            Fr::from(5u64),
            Fr::from(0xFFFFFFFF00000000u64),
        );
        let f3 = F3G::new(Fr::from(5u64), Fr::from(7u64), Fr::from(2u64));
        assert_eq!(f1 + f2, f3);
    }

    #[test]
    fn test_f3g_sub() {
        let f1 = F3G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64));
        let f2 = F3G::new(
            Fr::from(4u64),
            Fr::from(5u64),
            Fr::from(0xFFFFFFFF00000000u64),
        );
        let f3 = F3G::new(-Fr::from(3u64), -Fr::from(3u64), Fr::from(4u64));
        assert_eq!(f1 - f2, f3);
    }

    #[test]
    fn test_f3g_mul() {
        let f1 = F3G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64));
        let f2 = F3G::new(
            Fr::from(4u64),
            Fr::from(5u64),
            Fr::from(0xFFFFFFFF00000000u64),
        );
        let f3 = F3G::new(Fr::from(17u64), Fr::from(23u64), Fr::from(18u64));
        assert_eq!(f1 * f2, f3);
    }

    #[test]
    fn test_f3g_comparison() {
        let e1 = F3G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64));

        let elems = e1.as_elements();
        assert_eq!(elems[0], Fr::ONE);

        let e11 = F3G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64));

        let e12 = F3G::new(Fr::from(2u64), Fr::from(2u64), Fr::from(3u64));

        assert!(e1._eq(&e11));
        assert!(e1.lt(&e12));
    }

    #[test]
    fn test_f3g_exp() {
        let e1 = F3G::new(Fr::from(5u64), Fr::from(6u64), Fr::from(7u64));

        let expected = F3G::new(
            Fr::from(9897124412254467696u64),
            Fr::from(14730484130337994984u64),
            Fr::from(4476495173063158826u64),
        );

        assert!(e1.exp(100)._eq(&expected));
    }

    #[test]
    fn test_f3g_inv() {
        let mut rng = thread_rng();
        let tmp = F3G::random(&mut rng);
        let inv_tmp = tmp.inv();
        assert_eq!(tmp * inv_tmp, F3G::ONE);
    }

    #[test]
    fn test_f3g_batch_inverse() {
        let arr = vec![
            F3G::from(Fr::from(5u64)),
            F3G::from(Fr::from(6u64)),
            F3G::new(Fr::from(7u64), Fr::from(8u64), Fr::from(9u64)),
        ];
        let r_arr = batch_inverse(&arr);
        for i in 0..arr.len() {
            log::trace!("{} {}", arr[i].inv(), r_arr[i]);
            assert!(arr[i].inv()._eq(&r_arr[i]));
        }
    }

    #[test]
    fn test_f3g_inv3() {
        let a = F3G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64));
        let b = a.inv();
        let c = a.mul(b);
        assert_eq!(c, F3G::new(Fr::from(1), Fr::from(0), Fr::from(0)));
    }

    #[test]
    fn test_mul_simple_cases() {
        let x = F3G::new(Fr::from(1), Fr::from(0), Fr::from(0));
        let y = F3G::new(Fr::from(0), Fr::from(1), Fr::from(0));
        assert_eq!(x.mul(y), F3G::new(Fr::from(0), Fr::from(1), Fr::from(0)));
    }

    #[test]
    fn test_f3g_is_zero() {
        let a = &F3G::new(Fr::ZERO, Fr::ZERO, Fr::ZERO);
        let b = a.is_zero_vartime();
        assert!(b);

        let a = &F3G::new(Fr::ZERO, Fr::ONE, Fr::ZERO);
        let b = a.is_zero_vartime();
        assert!(!b);

        let a = &F3G::from(Fr::ZERO);
        let b = a.is_zero_vartime();
        assert!(b);

        let a = &F3G::from(Fr::ONE);
        let b = a.is_zero_vartime();
        assert!(!b);
    }
}
