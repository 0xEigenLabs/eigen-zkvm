#![allow(dead_code)]
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use plonky::field_gl::Fr;
use plonky::Field;
use rand::Rand;
use std::hash::{Hash, Hasher};
use std::slice;

use core::fmt::{Display, Formatter};
/// GF(2^3) implementation
/// Prime: 0xFFFFFFFF00000001
/// Irreducible polynomial: x^3 - x -1
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct F3G {
    pub cube: [Fr; 3],
    pub dim: usize,
}

impl Hash for F3G {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
        self.dim.hash(state);
    }
}

impl F3G {
    pub fn new(a: Fr, b: Fr, c: Fr) -> Self {
        F3G {
            cube: [a, b, c],
            dim: 3,
        }
    }

    pub const ZERO3: Self = Self {
        cube: [Fr::ZERO, Fr::ZERO, Fr::ZERO],
        dim: 3,
    };
    pub const ONE3: Self = Self {
        cube: [Fr::ONE, Fr::ZERO, Fr::ZERO],
        dim: 3,
    };

    #[inline(always)]
    pub fn to_be(&self) -> Fr {
        assert_eq!(self.dim, 1);
        self.as_elements()[0]
    }

    #[inline(always)]
    pub fn as_elements(&self) -> Vec<Fr> {
        let elements = &[self.cube];
        let ptr = elements.as_ptr();
        let len = elements.len() * self.dim;
        let elems: &[Fr] = unsafe { slice::from_raw_parts(ptr as *const Fr, len) };
        elems.to_vec()
    }

    #[inline]
    pub fn double(self) -> Self {
        self + self
    }

    #[inline]
    pub fn mul_scalar(self, b: usize) -> Self {
        let b = Fr::from(b as u64);
        let elems = self.as_elements();
        if self.dim == 1 {
            Self::from(elems[0] * b)
        } else {
            Self::new(elems[0] * b, elems[1] * b, elems[2] * b)
        }
    }

    #[inline]
    pub fn square(self) -> Self {
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
                    cube: [cc + gg - ff, aa + cc - ee - ee - dd, bb - gg],
                    dim: 3,
                }
            }
            1 => {
                let mut tmp = self.to_be();
                tmp.square();
                Self {
                    cube: [tmp, Fr::ZERO, Fr::ZERO],
                    dim: 1,
                }
            }
            _ => {
                panic!("Invalid dim");
            }
        }
    }

    #[inline]
    fn eq(self, rhs: &Self) -> bool {
        if self.dim == rhs.dim {
            self.cube == rhs.cube
        } else {
            if self.dim == 1 {
                self.cube[0] == rhs.cube[0] && rhs.cube[1] == Fr::ZERO && rhs.cube[2] == Fr::ZERO
            } else {
                self.cube[0] == rhs.cube[0]
                    || (self.cube[1] == Fr::ZERO)
                    || (self.cube[2] == Fr::ZERO)
            }
        }
    }

    #[inline]
    pub fn gt(self, rhs: &Self) -> bool {
        assert_eq!(self.dim, rhs.dim); // FIXME: align with JS
        let les = self.as_elements();
        let res = rhs.as_elements();
        match self.dim {
            3 => {
                (les[0] > res[0])
                    || ((les[0] == res[0]) && (les[1] == res[1]))
                    || ((les[0] == res[0]) && (les[1] == res[1]) && (les[2] > res[2]))
            }
            1 => les[0] > res[0],
            _ => {
                panic!("Invalid dim");
            }
        }
    }

    #[inline]
    pub fn geq(self, rhs: &Self) -> bool {
        self.eq(rhs) || self.gt(rhs)
    }

    #[inline]
    pub fn lt(self, rhs: &Self) -> bool {
        !self.geq(rhs)
    }

    #[inline]
    pub fn leq(self, rhs: &Self) -> bool {
        !self.gt(rhs)
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        match self.dim {
            1 => self.eq(&Self::ZERO),
            _ => self.eq(&Self::ZERO3),
        }
    }

    pub fn random() -> Self {
        let mut rng = ::rand::thread_rng();
        Self::from(Fr::rand(&mut rng))
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
            res = res.square();
            if bits[i] == 1 {
                res = res.mul(self);
            }
        }
        res
    }

    #[inline]
    pub fn pow(self, e: usize) -> Self {
        self.exp(e)
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

impl AddAssign for F3G {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
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

impl SubAssign for F3G {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
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
                    {
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
            }
            1 => {
                if rhs.dim == 1 {
                    Self::from(self.to_be() * rhs.to_be())
                } else {
                    //1 * 3
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
        *self = *self * rhs
    }
}

impl Div for F3G {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self * (rhs.inv())
    }
}

impl DivAssign for F3G {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl Neg for F3G {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        match self.dim {
            3 => Self {
                cube: [-self.cube[0], -self.cube[1], -self.cube[2]],
                dim: 3,
            },
            1 => Self::from(-self.to_be()),
            _ => {
                panic!("Invalid dim");
            }
        }
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

// FIXME
impl From<u128> for F3G {
    /// Converts a 128-bit value into a field element.
    fn from(_: u128) -> Self {
        //const R3: u128 = 1 (= 2^192 mod M );// thus we get that mont_red_var((mont_red_var(x) as u128) * R3) becomes
        //Self(mont_red_var(mont_red_var(x) as u128))  // Variable time implementation
        //Self(mont_red_cst(mont_red_cst(x) as u128)) // Constant time implementation
        panic!("Unimplement");
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
    pub const ZERO: Self = Self {
        cube: [Fr::ZERO, Fr::ZERO, Fr::ZERO],
        dim: 1,
    };
    pub const ONE: Self = Self {
        cube: [Fr::ONE, Fr::ZERO, Fr::ZERO],
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

    pub fn inv(self) -> Self {
        match self.dim {
            3 => {
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
                let tinv = t.inverse().unwrap();

                let i1 = (-aa - ac - ac + bc + bb - cc) * tinv;
                let i2 = (ba - cc) * tinv;
                let i3 = (-bb + ac + cc) * tinv;

                Self {
                    cube: [i1, i2, i3],
                    dim: 3,
                }
            }
            1 => Self::from(self.to_be().inverse().unwrap()),
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

impl Display for F3G {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let elems = self.as_elements();
        if self.dim == 1 {
            write!(f, "{}", elems[0].as_int())
        } else {
            write!(
                f,
                "[{},{},{}]",
                elems[0].as_int(),
                elems[1].as_int(),
                elems[2].as_int()
            )
        }
    }
}

impl F3G {
    fn as_bytes(&self) -> &[u8] {
        let self_ptr: *const Self = self;
        unsafe { slice::from_raw_parts(self_ptr as *const u8, Self::ELEMENT_BYTES * self.dim) }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::f3g::F3G;
    use plonky::field_gl::Fr;
    use plonky::to_hex;
    use plonky::Field;
    use plonky::PrimeField;
    use std::ops::{Add, Mul};

    #[test]
    fn test_f3g_add() {
        let f1 = F3G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64));
        let f2 = f1.add(f1);

        let f22 = f1.double();
        assert_eq!(f2, f22);

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

        assert_eq!(e1.eq(&e11), true);
        assert_eq!(e1.lt(&e12), true);
    }

    #[test]
    fn test_f3g_exp() {
        let e1 = F3G::new(Fr::from(5u64), Fr::from(6u64), Fr::from(7u64));

        let expected = F3G::new(
            Fr::from(9897124412254467696u64),
            Fr::from(14730484130337994984u64),
            Fr::from(4476495173063158826u64),
        );

        assert_eq!(e1.exp(100).eq(&expected), true);
    }

    #[test]
    fn test_f3g_inv() {
        let tmp = F3G::random();
        let inv_tmp = tmp.inv();
        assert_eq!(tmp * inv_tmp, F3G::ONE);
    }

    #[test]
    fn test_f3g_batch_inverse() {
        let arr = vec![
            F3G::from(5u64),
            F3G::from(6u64),
            F3G::new(Fr::from(7u64), Fr::from(8u64), Fr::from(9u64)),
        ];
        let r_arr = F3G::batch_inverse(&arr);
        for i in 0..arr.len() {
            println!("{} {}", arr[i].inv(), r_arr[i]);
            assert_eq!(arr[i].inv().eq(&r_arr[i]), true);
        }
    }

    #[test]
    fn test_f3g_inv3() {
        let a = F3G::new(Fr::ONE, Fr::from(2u64), Fr::from(3u64));

        let b = a.inv();
        let c = a.mul(b);
        assert_eq!(c, F3G::ONE3);
    }
}
