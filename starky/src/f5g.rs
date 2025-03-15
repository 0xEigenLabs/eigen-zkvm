#![allow(dead_code)]
use crate::traits::FieldExtension;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use fields::field_gl::Fr;
use fields::Field;
use std::hash::{Hash, Hasher};
use std::slice;

use core::fmt::{Display, Formatter};

/// Prime: 0xFFFFFFFF00000001
/// Irreducible polynomial: x^5-3
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct F5G {
    pub cube: [Fr; 5],
    pub dim: usize,
}

impl Hash for F5G {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
        self.dim.hash(state);
    }
}

impl F5G {
    pub fn new(a: Fr, b: Fr, c: Fr, d: Fr, e: Fr) -> Self {
        F5G { cube: [a, b, c, d, e], dim: 5 }
    }
}

/// Number of bytes needed to represent field element
const ELEMENT_BYTES: usize = core::mem::size_of::<u64>();

impl FieldExtension for F5G {
    const ELEMENT_BYTES: usize = ELEMENT_BYTES;
    const IS_CANONICAL: bool = false;

    const ZERO: Self = Self { cube: [Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO], dim: 1 };
    const ONE: Self = Self { cube: [Fr::ONE, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO], dim: 1 };
    const ZEROS: Self = F5G { cube: [Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO], dim: 5 };
    const ONES: Self = F5G { cube: [Fr::ONE, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO], dim: 5 };
    #[inline(always)]
    fn dim(&self) -> usize {
        self.dim
    }

    #[inline(always)]
    fn from_vec(values: Vec<Fr>) -> Self {
        assert_eq!(values.len(), 5);
        Self { cube: [values[0], values[1], values[2], values[3], values[4]], dim: 5 }
    }

    #[inline(always)]
    fn to_be(&self) -> Fr {
        assert_eq!(self.dim, 1);
        self.as_elements()[0]
    }

    #[inline(always)]
    fn as_elements(&self) -> Vec<Fr> {
        let elements = &[self.cube];
        let ptr = elements.as_ptr();
        let len = elements.len() * self.dim;
        let elems: &[Fr] = unsafe { slice::from_raw_parts(ptr as *const Fr, len) };
        elems.to_vec()
    }

    #[inline]
    fn mul_scalar(&self, b: usize) -> Self {
        let b = Fr::from(b as u64);
        let elems = self.as_elements();
        if self.dim == 1 {
            Self::from(elems[0] * b)
        } else {
            Self::new(elems[0] * b, elems[1] * b, elems[2] * b, elems[3] * b, elems[4] * b)
        }
    }

    #[inline]
    fn _eq(&self, rhs: &Self) -> bool {
        if self.dim == rhs.dim {
            self.cube == rhs.cube
        } else if self.dim == 1 {
            self.cube[0] == rhs.cube[0]
                && rhs.cube[1] == Fr::ZERO
                && rhs.cube[2] == Fr::ZERO
                && rhs.cube[3] == Fr::ZERO
                && rhs.cube[4] == Fr::ZERO
        } else {
            self.cube[0] == rhs.cube[0]
                && (self.cube[1] == Fr::ZERO)
                && (self.cube[2] == Fr::ZERO)
                && (self.cube[3] == Fr::ZERO)
                && (self.cube[4] == Fr::ZERO)
        }
    }

    #[inline]
    fn gt(&self, rhs: &Self) -> bool {
        assert_eq!(self.dim, rhs.dim); // FIXME: align with JS
        let les = self.as_elements();
        let res = rhs.as_elements();
        match self.dim {
            5 => {
                (les[0] > res[0]
                    && (les[1] == res[1])
                    && (les[2] == res[2])
                    && (les[3] == res[3])
                    && (les[4] == res[4]))
                    || ((les[0] == res[0]) && (les[1] > res[1]))
                    || ((les[0] == res[0]) && (les[1] == res[1]) && (les[2] > res[2]))
                    || ((les[0] == res[0])
                        && (les[1] == res[1])
                        && (les[2] == res[2])
                        && (les[3] > res[3]))
                    || ((les[0] == res[0])
                        && (les[1] == res[1])
                        && (les[2] == res[2])
                        && (les[3] == res[3])
                        && (les[4] > res[4]))
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
        !self.gt(rhs) || self.lt(rhs)
    }

    #[inline]
    fn leq(&self, rhs: &Self) -> bool {
        !self.gt(rhs)
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
                bits.push(0)
            }
            e >>= 1;
        }

        if bits.is_empty() {
            return Self::ONE;
        }

        let mut res = *self;
        for i in (0..bits.len() - 1).rev() {
            res.square();
            if bits[i] == 1 {
                res = res.mul(*self);
            }
        }
        res
    }

    #[inline(always)]
    fn inv(&self) -> Self {
        match self.dim {
            5 => self._inv(),
            1 => Self::from(self.to_be().inverse().unwrap()),
            _ => {
                panic!("Invalid dim");
            }
        }
    }

    #[inline]
    fn as_int(&self) -> u64 {
        self.as_elements()[0].as_int()
    }

    #[inline]
    fn elements_as_bytes(elements: &[Self]) -> &[u8] {
        // TODO: take endianness into account.
        let p = elements.as_ptr();
        let len = elements.len() * Self::ELEMENT_BYTES;
        unsafe { slice::from_raw_parts(p as *const u8, len) }
    }

    fn as_bytes(&self) -> &[u8] {
        let self_ptr: *const Self = self;
        unsafe { slice::from_raw_parts(self_ptr as *const u8, Self::ELEMENT_BYTES * self.dim) }
    }
}

impl ::rand::Rand for F5G {
    fn rand<R: rand::Rng>(rng: &mut R) -> Self {
        Self::from(Fr::rand(rng))
    }
}

impl fields::Field for F5G {
    #[inline(always)]
    fn zero() -> Self {
        Self::ZEROS
    }

    #[inline(always)]
    fn one() -> Self {
        Self::ONES
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        match self.dim {
            1 => self._eq(&Self::ZERO),
            _ => self._eq(&Self::zero()),
        }
    }

    #[inline(always)]
    fn square(&mut self) {
        match self.dim {
            5 => {
                let a = self.cube;
                let d0 = a[0] * a[0]
                    + Fr::from(3) * (a[1] * a[4] + a[2] * a[3] + a[3] * a[2] + a[4] * a[1]);
                let d1 = a[0] * a[1]
                    + a[1] * a[0]
                    + Fr::from(3) * (a[2] * a[4] + a[3] * a[3] + a[4] * a[2]);
                let d2 = a[0] * a[2]
                    + a[1] * a[1]
                    + a[2] * a[0]
                    + Fr::from(3) * (a[3] * a[4] + a[4] * a[3]);
                let d3 = a[0] * a[3]
                    + a[1] * a[2]
                    + a[2] * a[1]
                    + a[3] * a[0]
                    + Fr::from(3) * (a[4] * a[4]);
                let d4 = a[0] * a[4] + a[1] * a[3] + a[2] * a[2] + a[3] * a[1] + a[4] * a[0];
                *self = F5G { cube: [d0, d1, d2, d3, d4], dim: 5 }
            }
            1 => {
                let mut tmp = self.to_be();
                tmp.square();
                *self = F5G { cube: [tmp, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO], dim: 1 }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
    }

    #[inline(always)]
    fn double(&mut self) {
        *self = *self + *self;
    }

    #[inline(always)]
    fn negate(&mut self) {
        *self = self.neg();
    }

    #[inline(always)]
    fn add_assign(&mut self, other: &Self) {
        *self += *other
    }

    #[inline(always)]
    fn sub_assign(&mut self, other: &Self) {
        *self -= *other;
    }

    #[inline(always)]
    fn mul_assign(&mut self, other: &Self) {
        *self *= *other;
    }

    #[inline(always)]
    fn inverse(&self) -> Option<Self> {
        Some(self.inv())
    }

    #[inline(always)]
    fn frobenius_map(&mut self, _power: usize) {
        panic!("frobenius_map is not supported for F5G.");
    }
}

// `F5G` must implement `std::fmt::Display` trait when implement `fields::Field` trait
impl Display for F5G {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let elems = self.as_elements();
        if self.dim == 1 {
            write!(f, "{}", elems[0].as_int())
        } else {
            write!(
                f,
                "[{},{},{},{},{}]",
                elems[0].as_int(),
                elems[1].as_int(),
                elems[2].as_int(),
                elems[3].as_int(),
                elems[4].as_int()
            )
        }
    }
}

impl Add for F5G {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        match self.dim {
            5 => {
                if rhs.dim == 5 {
                    Self {
                        cube: [
                            self.cube[0] + rhs.cube[0],
                            self.cube[1] + rhs.cube[1],
                            self.cube[2] + rhs.cube[2],
                            self.cube[3] + rhs.cube[3],
                            self.cube[4] + rhs.cube[4],
                        ],
                        dim: 5,
                    }
                } else {
                    let r = self.as_elements();
                    Self::new(r[0] + rhs.to_be(), r[1], r[2], r[3], r[4])
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() + rhs.to_be())
                } else {
                    let r = rhs.as_elements();
                    Self::new(r[0] + self.to_be(), r[1], r[2], r[3], r[4])
                }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
    }
}

impl AddAssign for F5G {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for F5G {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        match self.dim {
            5 => {
                if rhs.dim == 5 {
                    Self {
                        cube: [
                            self.cube[0] - rhs.cube[0],
                            self.cube[1] - rhs.cube[1],
                            self.cube[2] - rhs.cube[2],
                            self.cube[3] - rhs.cube[3],
                            self.cube[4] - rhs.cube[4],
                        ],
                        dim: 5,
                    }
                } else if rhs.dim == 1 {
                    let r = self.as_elements();
                    Self::new(r[0] - rhs.to_be(), r[1], r[2], r[3], r[4])
                } else {
                    panic!("")
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() - rhs.to_be())
                } else {
                    let r = rhs.as_elements();
                    Self::new(self.to_be() - r[0], -r[1], -r[2], -r[3], -r[4])
                }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
    }
}

impl SubAssign for F5G {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for F5G {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        match self.dim {
            5 => {
                if rhs.dim == 1 {
                    let lhs = rhs.to_be();
                    let r = self.as_elements();
                    Self::new(lhs * r[0], lhs * r[1], lhs * r[2], lhs * r[3], lhs * r[4])
                } else if rhs.dim == 5 {
                    let a = self.cube;
                    let b = rhs.cube;
                    let d0 = a[0] * b[0]
                        + Fr::from(3) * (a[1] * b[4] + a[2] * b[3] + a[3] * b[2] + a[4] * b[1]);
                    let d1 = a[0] * b[1]
                        + a[1] * b[0]
                        + Fr::from(3) * (a[2] * b[4] + a[3] * b[3] + a[4] * b[2]);
                    let d2 = a[0] * b[2]
                        + a[1] * b[1]
                        + a[2] * b[0]
                        + Fr::from(3) * (a[3] * b[4] + a[4] * b[3]);
                    let d3 = a[0] * b[3]
                        + a[1] * b[2]
                        + a[2] * b[1]
                        + a[3] * b[0]
                        + Fr::from(3) * (a[4] * b[4]);
                    let d4 = a[0] * b[4] + a[1] * b[3] + a[2] * b[2] + a[3] * b[1] + a[4] * b[0];

                    Self { cube: [d0, d1, d2, d3, d4], dim: 5 }
                } else {
                    panic!("Invalid F5G Dim: {:?}", rhs.dim)
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() * rhs.to_be())
                } else if rhs.dim == 5 {
                    let lhs = self.to_be();
                    let r = rhs.as_elements();
                    Self::new(lhs * r[0], lhs * r[1], lhs * r[2], lhs * r[3], lhs * r[4])
                } else {
                    panic!("Invalid F5G Dim: {:?}", rhs.dim)
                }
            }
            _ => {
                panic!("Invalid F5G Dim: {:?}", self.dim)
            }
        }
    }
}

impl MulAssign for F5G {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl Div for F5G {
    type Output = Self;
    #[allow(clippy::suspicious_arithmetic_impl)]
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self * (rhs.inv())
    }
}

impl DivAssign for F5G {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl Neg for F5G {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        match self.dim {
            3 => Self {
                cube: [-self.cube[0], -self.cube[1], -self.cube[2], -self.cube[3], -self.cube[4]],
                dim: 3,
            },
            1 => Self::from(-self.to_be()),
            _ => {
                panic!("Invalid F5G Dim: {:?}", self.dim)
            }
        }
    }
}

impl From<Fr> for F5G {
    #[inline]
    fn from(value: Fr) -> Self {
        F5G { cube: [value, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO], dim: 1 }
    }
}

impl From<u64> for F5G {
    #[inline]
    fn from(value: u64) -> Self {
        Self::from(Fr::from(value))
    }
}

impl From<i32> for F5G {
    #[inline]
    fn from(value: i32) -> Self {
        Self::from(Fr::from(value as u64))
    }
}

impl From<u32> for F5G {
    #[inline]
    fn from(value: u32) -> Self {
        Self::from(Fr::from(value as u64))
    }
}

impl From<usize> for F5G {
    #[inline]
    fn from(value: usize) -> Self {
        Self::from(Fr::from(value as u64))
    }
}

impl From<&i32> for F5G {
    #[inline]
    fn from(value: &i32) -> Self {
        Self::from(Fr::from(*value as u64))
    }
}

impl From<&usize> for F5G {
    #[inline]
    fn from(value: &usize) -> Self {
        Self::from(Fr::from(*value as u64))
    }
}

impl From<u16> for F5G {
    /// Converts a 16-bit value into a field element.
    #[inline]
    fn from(value: u16) -> Self {
        Self::from(value as u64)
    }
}

impl From<u8> for F5G {
    /// Converts an 8-bit value into a field element.
    #[inline]
    fn from(value: u8) -> Self {
        Self::from(value as u64)
    }
}

impl From<[u8; 8]> for F5G {
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

impl F5G {
    // Frobenius operator (raise this value to the power p).
    #[inline]
    fn frob1(&self) -> Self {
        // Since z^5 = 3 in the field, and p = 1 mod 5, we have:
        // (z^i)^p = 3^(i*floor(p/5))*z^i
        // The Frobenius operator is a field automorphism, so we just
        // have to multiply the coefficients by the right values.
        assert_eq!(self.dim, 5);
        let c0 = self.cube[0];
        let c1 = self.cube[1] * Fr::from(1041288259238279555u64); // 3^(floor(p/5))
        let c2 = self.cube[2] * Fr::from(15820824984080659046u64); // 3^(2*floor(p/5))
        let c3 = self.cube[3] * Fr::from(211587555138949697u64); // 3^(3*floor(p/5))
        let c4 = self.cube[4] * Fr::from(1373043270956696022u64); // 3^(4*floor(p/5））
        Self { cube: [c0, c1, c2, c3, c4], dim: 5 }
    }

    // Frobenius operator, twice (raise this value to the power p^2).
    #[inline]
    fn frob2(&self) -> Self {
        assert_eq!(self.dim, 5);
        let c0 = self.cube[0];
        let c1 = self.cube[1] * Fr::from(15820824984080659046u64); // 9^(floor(p/5))
        let c2 = self.cube[2] * Fr::from(1373043270956696022u64); // 9^(2*floor(p/5))
        let c3 = self.cube[3] * Fr::from(1041288259238279555u64); // 9^(3*floor(p/5))
        let c4 = self.cube[4] * Fr::from(211587555138949697u64); // 9^(4*floor(p/5））
        Self { cube: [c0, c1, c2, c3, c4], dim: 5 }
    }

    // Invert this element. If this value is zero, then zero is returned.
    // Inv() function refers to the implementation of ecgfp5: https://github.com/pornin/ecgfp5/blob/ce059c6d1e1662db437aecbf3db6bb67fe63c716/python/ecGFp5.py#L751
    #[inline]
    fn _inv(&self) -> Self {
        // We are using the method first described by Itoh and Tsujii.
        //
        // Let r = 1 + p + p^2 + p^3 + p^4.
        // We have: p^5 - 1 = (p - 1)*r
        // For x != 0, we then have:
        //   x^(p^5-1) = (x^r)^(p-1)
        // Since x^(p^5-1) = 1 (the group of invertible elements has
        // order p^5-1), obtain that x^r is a root of the polynomial
        // X^(p-1) - 1. However, all non-zero elements of GF(p) are
        // roots of X^(p-1) - 1, and there are p-1 non-zero elements in
        // GF(p), and a polynomial of degre p-1 cannot have more than
        // p-1 roots in a field. Therefore, the roots of X^(p-1) - 1
        // are _exactly_ the elements of GF(p). It follows that x^r is
        // in GF(p), for any x != 0 in GF(p^5) (this also holds for x = 0).
        //
        // Given x != 0, we can write:
        //   1/x = x^(r-1) / x^r
        // Thus, we only need to compute x^(r-1) (in GF(p^5)), then x^r
        // (by multiplying x with x^(r-1)), then invert x^r in GF(p),
        // and multiply x^(r-1) by the inverse of x^r.
        //
        // We can compute efficiently x^(r-1) by using the Frobenius
        // operator: in GF(p^5), raising a value to the power p boils
        // down to multiplying four of the coefficients by precomputed
        // constants.
        // If we defined phi1(x) = x^p and phi2(x) = phi1(phi1(x)), then:
        //   x^(r-1) = x^(p + p^2 + p^3 + p^4)
        //           = x^(p + p^2) * phi2(x^(p + p^2))
        //           = phi1(x) * phi1(phi1(x)) * phi2(phi1(x) * phi1(phi1(x)))
        // which only needs three applications of phi1() or phi2(), and
        // two multiplications in GF(p^5).

        // t0 <- a^p
        let t0 = self.frob1();

        // t1 <- a^(p + p^2)
        let t1 = t0.mul(t0.frob1());

        // t2 <- a^(p + p^2 + p^3 + p^4)
        let t2 = t1.mul(t1.frob2());

        //compute x^r =t2 * x
        let a = self.cube;
        let b = t2.cube;
        // Let r = 1 + p + p^2 + p^3 + p^4. We have a^r = a * t2. Also,
        // (a^r)^(p-1) = a^(p^5-1) = 1, so a^r is in GF(p) (b^(p-1) = 1 for
        // all non-zero elements in GF(p), and that's p-1 solutions to a
        // polynomial of degree p-1, so it works in the other direction too:
        // all values b such that b^(p-1) = 1 must be in GF(p)). Thus,
        // We can compute a^r as only the low coefficient of a*t2 (into t3).
        let mut t3 =
            a[0] * b[0] + Fr::from(3) * (a[1] * b[4] + a[2] * b[3] + a[3] * b[2] + a[4] * b[1]);
        if t3.is_zero() {
            // If input 'a' is zero then we will divide 0 by 0, which is not
            // defined; we need a small corrective step to make divisor t3
            // equal to 1 in that case (the final output will still be zero,
            // since in such a case t2 = (0,0,0,0,0)).
            t3 = Fr::ONE;
        }
        let t4 = t3.inverse().unwrap();
        t2.mul(Self::from(t4))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::f5g::F5G;
    use crate::polutils::batch_inverse;
    use crate::traits::FieldExtension;
    use fields::field_gl::Fr;
    use fields::Field;
    use std::ops::{Add, Mul};

    impl F5G {
        pub fn rand_gen() -> F5G {
            F5G::new(
                Fr::from(rand::random::<u64>()),
                Fr::from(rand::random::<u64>()),
                Fr::from(rand::random::<u64>()),
                Fr::from(rand::random::<u64>()),
                Fr::from(rand::random::<u64>()),
            )
        }
    }

    #[test]
    fn test_f5g_add() {
        let mut f1 =
            F5G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64), Fr::from(4u64), Fr::from(5u64));
        let f2 = f1.add(f1);

        f1.double();
        assert_eq!(f2, f1);

        let f1 = F5G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64), Fr::from(0u64), Fr::from(2u64));
        let f2 = F5G::new(
            Fr::from(4u64),
            Fr::from(5u64),
            Fr::from(0xFFFFFFFF00000000u64),
            Fr::from(0u64),
            Fr::from(2u64),
        );
        let f3 = F5G::new(
            Fr::from(5u64),
            Fr::from(7u64),
            Fr::from(2u64),
            Fr::from(0u64),
            Fr::from(4u64),
        );
        assert_eq!(f1 + f2, f3);

        let f1 = F5G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64), Fr::from(3u64), Fr::from(3u64));
        let f2 = F5G::new(
            Fr::from(4u64),
            Fr::from(5u64),
            Fr::from(0xFFFFFFFF00000000u64),
            Fr::from(0xFFFFFFFF00000000u64),
            Fr::from(0xFFFFFFFF00000000u64),
        );
        let f3 = F5G::new(
            Fr::from(5u64),
            Fr::from(7u64),
            Fr::from(2u64),
            Fr::from(2u64),
            Fr::from(2u64),
        );
        assert_eq!(f1 + f2, f3);
    }

    #[test]
    fn test_f5g_sub() {
        let f1 = F5G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64), Fr::from(4u64), Fr::from(5u64));
        let f2 = F5G::new(
            Fr::from(4u64),
            Fr::from(5u64),
            Fr::from(0xFFFFFFFF00000000u64),
            Fr::from(4u64),
            Fr::from(5u64),
        );
        let f3 = F5G::new(-Fr::from(3u64), -Fr::from(3u64), Fr::from(4u64), Fr::ZERO, Fr::ZERO);
        assert_eq!(f1 - f2, f3);

        let f1 = F5G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64), Fr::from(3u64), Fr::from(3u64));
        let f2 = F5G::new(
            Fr::from(4u64),
            Fr::from(5u64),
            Fr::from(0xFFFFFFFF00000000u64),
            Fr::from(0xFFFFFFFF00000000u64),
            Fr::from(0xFFFFFFFF00000000u64),
        );
        let f3 = F5G::new(
            -Fr::from(3u64),
            -Fr::from(3u64),
            Fr::from(4u64),
            Fr::from(4u64),
            Fr::from(4u64),
        );
        assert_eq!(f1 - f2, f3);
    }

    #[test]
    fn test_f5g_mul() {
        let a = F5G::new(
            Fr::from(9788683869780751860),
            Fr::from(18176307314149915536),
            Fr::from(17581807048943060475),
            Fr::from(16706651231658143014),
            Fr::from(424516324638612383),
        );
        let b = F5G::new(
            Fr::from(1541862605911742196),
            Fr::from(5168181287870979863),
            Fr::from(10854086836664484156),
            Fr::from(11043707160649157424),
            Fr::from(943499178011708365),
        );

        let atb = F5G::new(
            Fr::from(5924286846078684570),
            Fr::from(12564682493825924142),
            Fr::from(17116577152380521223),
            Fr::from(5260948460973948760),
            Fr::from(15673927150284637712),
        );

        assert_eq!(a * b, atb)
    }

    #[test]
    fn test_f5g_comparison() {
        let e1 = F5G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64), Fr::from(4u64), Fr::from(5u64));

        let elems = e1.as_elements();
        assert_eq!(elems[0], Fr::ONE);

        let e11 = F5G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64), Fr::from(4u64), Fr::from(5u64));

        let e12 = F5G::new(
            Fr::from(2u64),
            Fr::from(2u64),
            Fr::from(3u64),
            Fr::from(4u64),
            Fr::from(5u64),
        );

        assert!(e1._eq(&e11));
        assert!(e1.geq(&e11));

        assert!(e1.lt(&e12));
        assert!(e12.gt(&e1));
        assert!(e12.geq(&e1));
    }

    #[test]
    fn test_f5g_inv5() {
        let mut rng = ::rand::thread_rng();
        let tmp = <F5G as rand::Rand>::rand(&mut rng);
        let inv_tmp = tmp.inv();
        assert_eq!(tmp * inv_tmp, F5G::ONE);

        let a = F5G::new(
            Fr::from(1u64),
            Fr::from(2u64),
            Fr::from(3u64),
            Fr::from(4u64),
            Fr::from(5u64),
        );

        let inv_a = a.inv();
        let c = a.mul(inv_a);
        assert_eq!(c.cube, F5G::ONE.cube);

        let a = F5G::rand_gen();
        let inv_a = a.inv();
        let c = a.mul(inv_a);
        assert_eq!(c.cube, F5G::ONE.cube);

        // special case: a is equal to [0,0,0,0,0]
        let a = F5G::new(Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO);

        let inv_a = a.inv();
        assert_eq!(a, inv_a);

        // special case: a is equal to [1,0,0,0,0]
        let a = F5G::new(Fr::from(1u64), Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO);

        let inv_a = a.inv();
        assert_eq!(a, inv_a);
        assert_eq!(a.cube, F5G::ONE.cube);
    }

    #[test]
    fn test_f5g_batch_inverse() {
        let arr = vec![
            F5G::from(5u64),
            F5G::from(6u64),
            F5G::new(
                Fr::from(7u64),
                Fr::from(8u64),
                Fr::from(9u64),
                Fr::from(10u64),
                Fr::from(11u64),
            ),
            F5G::rand_gen(),
        ];
        let r_arr = batch_inverse(&arr);
        for i in 0..arr.len() {
            log::trace!("{} {}", arr[i].inv(), r_arr[i]);
            assert!(arr[i].inv()._eq(&r_arr[i]));
        }
    }

    #[test]
    fn test_f3g_is_zero() {
        let a = &F5G::new(Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO);
        let b = a.is_zero();
        assert!(b);

        let a = &F5G::new(Fr::ZERO, Fr::ONE, Fr::ZERO, Fr::ZERO, Fr::ZERO);
        let b = a.is_zero();
        assert!(!b);

        let a = &F5G::from(Fr::ZERO);
        let b = a.is_zero();
        assert!(b);

        let a = &F5G::from(Fr::ONE);
        let b = a.is_zero();
        assert!(!b);
    }
}
