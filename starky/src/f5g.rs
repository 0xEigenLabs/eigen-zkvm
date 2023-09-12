#![allow(dead_code)]
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::fmt::write;
use plonky::field_gl::Fr;
use plonky::Field;
use std::hash::{Hash, Hasher};
use std::slice;

use core::fmt::{Display, Formatter};
/// Irreducible polynomial: x5-3
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct F5G {
    pub cube: [Fr; 5],
    pub dim: usize,
}

impl Hash for F5G {
    fn hash<H:Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
        self.dim.hash(state);
    }
}

impl F5G {
    pub fn new(a: Fr, b: Fr, c: Fr , d:Fr, e:Fr) -> Self {
        F5G {
            cube: [a, b, c, d, e],
            dim: 5,
        }
    }

    #[inline(always)]
    pub fn to_be(&self) -> Fr {
        assert_eq!(self.dim, 1);
        self.as_elements()[0]
    }

    #[inline(always)]
    pub fn as_elements(&self) -> Vec<Fr> {
        // 创建了一个包含 self.cube 数组的引用的切片 elements
        let elements = &[self.cube];
        // 这一行获取了 elements 切片的指针，也就是 self.cube 数组的指针
        let ptr = elements.as_ptr();
        let len = elements.len() * self.dim;
        let elems: &[Fr] = unsafe { slice::from_raw_parts(ptr as *const Fr, len) };
        elems.to_vec()
    }

    #[inline]
    pub fn mul_scalar(self, b: usize) -> Self {
        let b = Fr::from(b as u64);
        let elems = self.as_elements();
        if self.dim == 1 {
            Self::from(elems[0] * b)
        }else {
            Self::new(elems[0] * b, elems[1] * b, elems[2] * b,elems[3] * b,elems[4] * b)
        }
    }

    #[inline]
    fn eq(self, rhs: &Self) -> bool {
        if self.dim == rhs.dim {
            self.cube == rhs.cube 
        } else {
            if self.dim == 1 {
                self.cube[0] == rhs.cube[0] && rhs.cube[1] == Fr::ZERO && rhs.cube[2] == Fr::ZERO && rhs.cube[3] == Fr::ZERO && rhs.cube[4] == Fr::ZERO 
            } else {
                self.cube[0] == rhs.cube[0]
                    || (self.cube[1] == Fr::ZERO)
                    || (self.cube[2] == Fr::ZERO)
                    || (self.cube[3] == Fr::ZERO)
                    || (self.cube[4] == Fr::ZERO)
            }
        }
    }

    #[inline]
    pub fn gt(self, rhs: &Self) -> bool {
        assert_eq!(self.dim, rhs.dim); // FIXME: align with JS
        let les = self.as_elements();
        let res = rhs.as_elements();
        match self.dim {
            5 => {
                (les[0] > res[0]&& (les[1] == res[1])&& (les[2] == res[2])&& (les[3] == res[3]) && (les[4] == res[4]))
                    || ((les[0] == res[0]) && (les[1] > res[1])&& (les[2] == res[2])&& (les[3] == res[3]) && (les[4] == res[4]))
                    || ((les[0] == res[0]) && (les[1] == res[1]) && (les[2] > res[2])&& (les[3] == res[3]) && (les[4] == res[4]))
                    || ((les[0] == res[0]) && (les[1] == res[1]) && (les[2] == res[2]) && (les[3] > res[3]) && (les[4] == res[4]))
                    || ((les[0] == res[0]) && (les[1] == res[1]) && (les[2] == res[2]) && (les[3] == res[3]) && (les[4] > res[4]))
            }
            1 => les[0] > res[0],
            _ => {
                panic!("Invalid dim");
            }
        }
    }

    #[inline]
    pub fn geq(self, rhs :&Self)-> bool {
        self.eq(rhs) || self.gt(rhs)
    }

    #[inline]
    pub fn lt(self, rhs: &Self)-> bool {
        !self.gt(rhs) || self.lt(rhs)
    }

    #[inline]
    pub fn leq(self, rhs: &Self) -> bool {
        !self.gt(rhs)
    }

    #[inline]
    pub fn exp(self, e_: usize) -> Self {
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
            e = e >> 1;
        }

        if bits.len() == 0 {
            return Self::ONE;
        }

        let mut res = self;
        for i in (0..bits.len() - 1).rev() {
            res.square();
            if bits[i] == 1 {
                res = res.mul(self);
            }
        }
        res
    }

    #[inline]
    pub fn batch_inverse(elems: &[Self]) -> Vec<Self> {
        if elems.len() == 0 {
            return vec![];
        }

        let mut tmp: Vec<Self> = vec![Self::ZERO; elems.len()];
        tmp[0] = elems[0];
        for i in 1..elems.len() {
            tmp[i] = elems[i] * (tmp[i - 1]);
        }
        let mut z = tmp[tmp.len() - 1].inv();
        let mut res: Vec<Self> = vec![Self::ZERO; elems.len()];
        for i in (1..elems.len()).rev() {
            res[i] = z * tmp[i - 1];
            z = z * elems[i];
        }
        res[0] = z;
        res
    }
}

impl ::rand::Rand for F5G {
    fn rand<R: rand::Rng>(rng: &mut R) -> Self {
        Self::from(Fr::rand(rng))
    }
}

impl plonky::Field for F5G {
    #[inline(always)]
    fn zero() -> Self {
        F5G {
            cube: [Fr::ONE, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO],
            dim: 5,
        }
    }

    #[inline(always)]
    fn one() -> Self {
        F5G {
            cube: [Fr::ONE, Fr::ZERO, Fr::ZERO,  Fr::ZERO, Fr::ZERO],
            dim: 5,
        }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        match self.dim {
            1 => self.eq(&Self::ZERO),
            _ => self.eq(&Self::zero()),
        }
    }

    #[inline(always)]
    fn square(&mut self) {
        match self.dim {
            5 => {
                let a = self.cube;
                let d0 = a[0]*a[0] + Fr::from(3)*(a[1]*a[4] + a[2]*a[3] + a[3]*a[2] + a[4]*a[1]);
                let d1 = a[0]*a[1] + a[1]*a[0] + Fr::from(3)*( a[2]*a[4] + a[3]*a[3] + a[4]*a[2]);
                let d2 = a[0]*a[2] + a[1]*a[1] + a[2]*a[0] + Fr::from(3)*(a[3]*a[4] + a[4]*a[3]);
                let d3 = a[0]*a[3] + a[1]*a[2] + a[2]*a[1] + a[3]*a[0] + Fr::from(3)*(a[4]*a[4]);
                let d4 = a[0]*a[4] + a[1]*a[3] + a[2]*a[2] + a[3]*a[1] + a[4]*a[0];
                *self = F5G {
                    cube: [d0,d1,d2,d3,d4],
                    dim: 5,
                }
            }
            1 => {
                let mut tmp = self.to_be();
                tmp.square();
                *self = F5G {
                    cube: [tmp, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO],
                    dim: 1,
                }
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
        *self = *self + *other
    }

    #[inline(always)]
    fn sub_assign(&mut self, other: &Self) {
        *self = *self - *other;
    }

    #[inline(always)]
    fn mul_assign(&mut self, other: &Self) {
        *self = *self * *other;
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

// `F5G` must implement `std::fmt::Display` trait when implement `plonky::Field` trait 
impl Display for F5G {
    fn fmt(&self, f: &mut  Formatter) -> core::fmt::Result {
        let elems = self.as_elements();
        if self.dim == 1 {
            write!(f,"{}", elems[0].as_int())
        }else {
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
                    Self::new(r[0] + rhs.to_be(), r[1], r[2],r[3],r[4])
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() + rhs.to_be())
                } else {
                    let r = rhs.as_elements();
                    Self::new(r[0] + self.to_be(), r[1], r[2],r[3],r[4])
                }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
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
                    Self::new(r[0] - rhs.to_be(), r[1], r[2], r[3] , r[4])
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
                }else if rhs.dim == 5{
                    let a = self.cube;
                    let b = rhs.cube;
                    let d0 = a[0]*b[0] + Fr::from(3)*(a[1]*b[4] + a[2]*b[3] + a[3]*b[2] + a[4]*b[1]) ;
                    let d1 = a[0]*b[1] + a[1]*b[0] + Fr::from(3)*( a[2]*b[4] + a[3]*b[3] + a[4]*b[2]) ;
                    let d2 = a[0]*b[2] + a[1]*b[1] + a[2]*b[0] + Fr::from(3)*(a[3]*b[4] + a[4]*b[3]) ;
                    let d3 = a[0]*b[3] + a[1]*b[2] + a[2]*b[1] + a[3]*b[0] + Fr::from(3)*(a[4]*b[4]) ;
                    let d4 = a[0]*b[4] + a[1]*b[3] + a[2]*b[2] + a[3]*b[1] + a[4]*b[0];

                    Self {
                        cube: [d0,d1,d2,d3,d4],
                        dim: 5,
                    }
                }else {
                    panic!("Invalid F5G Dim: {:?}", rhs.dim)
                }
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() * rhs.to_be())
                }else if rhs.dim == 5 {
                    let lhs = self.to_be();
                    let r = rhs.as_elements();
                    Self::new(lhs * r[0], lhs * r[1], lhs * r[2],lhs * r[3],lhs * r[4])
                }else {
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

impl Neg for F5G {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        match self.dim {
            3 => Self {
                cube: [-self.cube[0], -self.cube[1], -self.cube[2],-self.cube[3],-self.cube[4]],
                dim: 3,
            },
            1 => Self::from(-self.to_be()),
            _ => {
                panic!("Invalid F5G Dim: {:?}", self.dim)
            }
        }
    }
}

impl From<Fr> for F5G{
    #[inline]
    fn from(value: Fr) -> Self {
        F5G{
            cube: [value,Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO],
            dim: 1,
        }
    }
}

impl From<u64> for F5G {
    #[inline]
    fn from (value: u64) -> Self {
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

// FIXME
impl From<u128> for F5G {
    /// Converts a 128-bit value into a field element.
    fn from(_: u128) -> Self {
        //const R3: u128 = 1 (= 2^192 mod M );// thus we get that mont_red_var((mont_red_var(x) as u128) * R3) becomes
        //Self(mont_red_var(mont_red_var(x) as u128))  // Variable time implementation
        //Self(mont_red_cst(mont_red_cst(x) as u128)) // Constant time implementation
        panic!("Unimplement");
    }
}

/// Number of bytes needed to represent field element
const ELEMENT_BYTES: usize = core::mem::size_of::<u64>();

impl F5G {
    pub const ZERO: Self = Self {
        cube: [Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO],
        dim: 1,
    };
    pub const ONE: Self = Self {
        cube: [Fr::ONE, Fr::ZERO, Fr::ZERO, Fr::ZERO, Fr::ZERO],
        dim: 1,
    };

    const ELEMENT_BYTES: usize = ELEMENT_BYTES;
    const IS_CANONICAL: bool = false;

    #[inline]
    pub fn as_int(&self) -> u64 {
        /*
        if self.dim == 1 {
            self.to_be().as_int()
        } else {
            panic!("Invalid as int: {:?}", *self);
        }
        */
        self.as_elements()[0].as_int()
    }

     // Frobenius operator (raise this value to the power p).
    #[inline]
    fn frob1(self) -> Self{
        // Since z^5 = 3 in the field, and p = 1 mod 5, we have:
        // (z^i)^p = 3^(i*floor(p/5))*z^i
        // The Frobenius operator is a field automorphism, so we just
        // have to multiply the coefficients by the right values.
        assert!(self.dim==5);
        let c0 = self.cube[0];
        let c1 = self.cube[1] * Fr::from( 1041288259238279555); // # 3^(floor(p/5))
        let c2 = self.cube[2] * Fr::from(15820824984080659046); // # 3^(2*floor(p/5))
        let c3 = self.cube[3] * Fr::from(  211587555138949697); // # 3^(3*floor(p/5))
        let c4 = self.cube[4] * Fr::from( 1373043270956696022); // # 3^(4*floor(p/5））    
        Self { cube: [c0,c1,c2,c3,c4], dim: 5 }
    }

    // Frobenius operator, twice (raise this value to the power p^2).
    #[inline]
    fn frob2(self) -> Self{
        assert!(self.dim==5);
        let c0 = self.cube[0];
        let c1 = self.cube[1] * Fr::from( 15820824984080659046); // # 9^(floor(p/5))
        let c2 = self.cube[2] * Fr::from(1373043270956696022); // # 9^(2*floor(p/5))
        let c3 = self.cube[3] * Fr::from(  1041288259238279555); // # 9^(3*floor(p/5))
        let c4 = self.cube[4] * Fr::from( 211587555138949697); // # 9^(4*floor(p/5））
        Self { cube: [c0,c1,c2,c3,c4], dim: 5 }
    }

    /// Invert this element. If this value is zero, then zero is returned.
    pub(crate) fn inv(self) -> Self{
        match self.dim {
            5 => {
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
                
                // t0 = self.frob1()         # t0 <- x^p
                // t1 = t0 * t0.frob1()      # t1 <- x^(p + p^2)
                // t2 = t1 * t1.frob2()      # t2 <- x^(p + p^2 + p^3 + p^4)
                let t0 = self.frob1(); // t0 = a^p 

                let t1 = t0.frob1().mul(t0); 

                let t2 = t1.mul(t1.frob2());

                //compute x^r =t2 * x  
                let mut t3 = t2.mul(self);
                // we need to confirm that the t3 can not be zero
                if t3.is_zero() {
                    t3 = Self::ONE;
                }
                let t4 = t3.inv();
                t4
            }
            1 => { 
                Self::from(self.to_be().inverse().unwrap())
            }
            _ => {
                panic!("Invalid dim");
            }
        }

    }

    pub fn elements_as_bytes(elements: &[Self]) -> &[u8] {
        // TODO: take endianness into account.
        let p = elements.as_ptr();
        let len = elements.len() * Self::ELEMENT_BYTES;
        unsafe { slice::from_raw_parts(p as *const u8, len) }
    }

}

impl F5G {
    fn as_bytes(&self) -> &[u8] {
        let self_ptr: *const Self = self;
        unsafe { slice::from_raw_parts(self_ptr as *const u8, Self::ELEMENT_BYTES * self.dim) }
    }
}


#[cfg(test)]
pub mod tests {
    use crate::f5g::F5G;
    use plonky::field_gl::Fr;
    use plonky::Field;
    use std::ops::{Add, Mul};

    #[test]
    fn test_f5g_add() {
        let mut f1 = F5G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64),Fr::from(4u64),Fr::from(5u64));
        let f2 = f1.add(f1);

        f1.double();
        assert_eq!(f2, f1);

        let f1 = F5G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64),Fr::from(0u64),Fr::from(2u64));
        let f2 = F5G::new(
            Fr::from(4u64),
            Fr::from(5u64),
            Fr::from(0xFFFFFFFF00000000u64),
            Fr::from(0u64),
            Fr::from(2u64)
        );
        let f3 = F5G::new(Fr::from(5u64), Fr::from(7u64), Fr::from(2u64),Fr::from(0u64),Fr::from(4u64));
        assert_eq!(f1 + f2, f3);

        let f1 = F5G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64),Fr::from(3u64),Fr::from(3u64));
        let f2 = F5G::new(
            Fr::from(4u64),
            Fr::from(5u64),
            Fr::from(0xFFFFFFFF00000000u64),
            Fr::from(0xFFFFFFFF00000000u64),
            Fr::from(0xFFFFFFFF00000000u64),
        );
        let f3 = F5G::new(Fr::from(5u64), Fr::from(7u64), Fr::from(2u64),Fr::from(2u64),Fr::from(2u64));
        assert_eq!(f1 + f2, f3);
    }
}