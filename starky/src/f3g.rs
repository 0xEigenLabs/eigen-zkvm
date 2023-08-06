#![allow(dead_code)]
use core::mem;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use rand_utils::rand_vector;
use std::hash::{Hash, Hasher};
use std::slice;
use winter_math::fields::f64::BaseElement;
use winter_math::fields::CubeExtension;
use winter_math::{FieldElement, StarkField};
use winter_utils::{
    AsBytes, ByteReader, ByteWriter, Deserializable, DeserializationError, Randomizable,
    Serializable,
};

use core::fmt::{Display, Formatter};
/// GF(2^3) implementation
/// Prime: 0xFFFFFFFF00000001
/// Irreducible polynomial: x^3 - x -1
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct F3G {
    cube: CubeExtension<BaseElement>,
    pub dim: i32,
}

impl Hash for F3G {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cube.as_bytes().hash(state);
        self.dim.hash(state);
    }
}

impl F3G {
    pub fn new(a: BaseElement, b: BaseElement, c: BaseElement) -> Self {
        F3G {
            cube: CubeExtension::<BaseElement>::new(a, b, c),
            dim: 3,
        }
    }

    pub const ZERO3: Self = Self {
        cube: CubeExtension::<BaseElement>::ZERO,
        dim: 3,
    };
    pub const ONE3: Self = Self {
        cube: CubeExtension::<BaseElement>::ONE,
        dim: 3,
    };

    #[inline(always)]
    pub fn to_be(&self) -> BaseElement {
        assert_eq!(self.dim, 1);
        let cc = &[self.cube];
        let elems = CubeExtension::<BaseElement>::as_base_elements(cc).to_vec();
        elems[0]
    }

    #[inline(always)]
    pub fn as_elements(&self) -> Vec<BaseElement> {
        let cc = &[self.cube];
        let elems = CubeExtension::<BaseElement>::as_base_elements(cc).to_vec();
        if self.dim == 3 {
            elems
        } else {
            elems[..1].to_vec()
        }
    }

    #[inline]
    pub fn double(self) -> Self {
        self + self
    }

    #[inline]
    pub fn mul_scalar(self, b: usize) -> Self {
        let b = BaseElement::from(b as u128);
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
            3 => Self {
                cube: self.cube.square(),
                dim: 3,
            },
            1 => Self::from(self.to_be().square()),
            _ => {
                panic!("Invalid dim");
            }
        }
    }

    #[inline]
    fn eq(self, rhs: &Self) -> bool {
        assert_eq!(self.dim, rhs.dim); // FIXME: align with JS
        self.cube == rhs.cube
    }

    #[inline]
    pub fn gt(self, rhs: &Self) -> bool {
        assert_eq!(self.dim, rhs.dim); // FIXME: align with JS
        let les = self.as_elements();
        let res = rhs.as_elements();
        match self.dim {
            3 => {
                (les[0].as_int() > res[0].as_int())
                    || ((les[0].as_int() == res[0].as_int())
                        && (les[1].as_int() == res[1].as_int()))
                    || ((les[0].as_int() == res[0].as_int())
                        && (les[1].as_int() == res[1].as_int())
                        && (les[2].as_int() > res[2].as_int()))
            }
            1 => les[0].as_int() > res[0].as_int(),
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
        let cube = rand_vector::<BaseElement>(1);
        Self::from(cube[0])
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
        winter_math::batch_inversion(elems)

        /*
        if elems.len() == 0 {
            return vec![];
        }

        let mut tmp: Vec<Self> = vec![Self::ZERO; elems.len()];
        tmp[0] = elems[0];
        for i in 1..elems.len() {
            tmp[i] = elems[i].mul(tmp[i - 1]);
        }
        let mut z = tmp[tmp.len() - 1].inv();
        let mut res: Vec<Self> = vec![Self::ZERO; elems.len()];
        for i in (1..elems.len()).rev() {
            res[i] = z.mul(tmp[i - 1]);
            z = z.mul(elems[i]);
        }
        res[0] = z;
        res
        */
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
                        cube: self.cube.add(rhs.cube),
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
                        cube: self.cube.sub(rhs.cube),
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
                    Self {
                        cube: self.cube.mul(rhs.cube),
                        dim: 3,
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
                cube: self.cube.neg(),
                dim: 3,
            },
            1 => Self::from(-self.to_be()),
            _ => {
                panic!("Invalid dim");
            }
        }
    }
}

impl From<BaseElement> for F3G {
    #[inline]
    fn from(value: BaseElement) -> Self {
        F3G {
            cube: CubeExtension::<BaseElement>::new(value, BaseElement::ZERO, BaseElement::ZERO),
            dim: 1,
        }
    }
}

impl From<u64> for F3G {
    #[inline]
    fn from(value: u64) -> Self {
        Self::from(BaseElement::from(value))
    }
}

impl From<i32> for F3G {
    #[inline]
    fn from(value: i32) -> Self {
        Self::from(BaseElement::from(value as u64))
    }
}

impl From<u32> for F3G {
    #[inline]
    fn from(value: u32) -> Self {
        Self::from(BaseElement::from(value as u64))
    }
}

impl From<usize> for F3G {
    #[inline]
    fn from(value: usize) -> Self {
        Self::from(BaseElement::from(value as u64))
    }
}

impl From<&i32> for F3G {
    #[inline]
    fn from(value: &i32) -> Self {
        Self::from(BaseElement::from(*value as u64))
    }
}

impl From<&usize> for F3G {
    #[inline]
    fn from(value: &usize) -> Self {
        Self::from(BaseElement::from(*value as u64))
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

impl FieldElement for F3G {
    type PositiveInteger = u64;
    type BaseField = Self;

    const ZERO: Self = Self {
        cube: CubeExtension::<BaseElement>::ZERO,
        dim: 1,
    };
    const ONE: Self = Self {
        cube: CubeExtension::<BaseElement>::ONE,
        dim: 1,
    };

    const ELEMENT_BYTES: usize = ELEMENT_BYTES;
    const IS_CANONICAL: bool = false;

    fn inv(self) -> Self {
        match self.dim {
            3 => Self {
                cube: self.cube.inv(),
                dim: 3,
            },
            1 => Self::from(self.to_be().inv()),
            _ => {
                panic!("Invalid dim");
            }
        }
    }

    fn conjugate(&self) -> Self {
        panic!("Unimplement");
    }

    fn elements_as_bytes(elements: &[Self]) -> &[u8] {
        // TODO: take endianness into account.
        let p = elements.as_ptr();
        let len = elements.len() * Self::ELEMENT_BYTES;
        unsafe { slice::from_raw_parts(p as *const u8, len) }
    }

    unsafe fn bytes_as_elements(bytes: &[u8]) -> Result<&[Self], DeserializationError> {
        if bytes.len() % Self::ELEMENT_BYTES != 0 {
            return Err(DeserializationError::InvalidValue(format!(
                "number of bytes ({}) does not divide into whole number of field elements",
                bytes.len(),
            )));
        }

        let p = bytes.as_ptr();
        let len = bytes.len() / Self::ELEMENT_BYTES;

        if (p as usize) % mem::align_of::<u64>() != 0 {
            return Err(DeserializationError::InvalidValue(
                "slice memory alignment is not valid for this field element type".to_string(),
            ));
        }

        Ok(slice::from_raw_parts(p as *const Self, len))
    }

    fn as_base_elements(elements: &[Self]) -> &[Self::BaseField] {
        elements
    }
}

// FIXME
impl Randomizable for F3G {
    const VALUE_SIZE: usize = Self::ELEMENT_BYTES;

    fn from_random_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(bytes).ok()
    }
}

// FIXME
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

// FIXME
impl Serializable for F3G {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        // convert from Montgomery representation into canonical representation
        target.write_u8_slice(&self.as_int().to_le_bytes());
    }
}

// FIXME
impl Deserializable for F3G {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let value = source.read_u64()?;
        if value >= M {
            return Err(DeserializationError::InvalidValue(format!(
                "invalid field element: value {} is greater than or equal to the field modulus",
                value
            )));
        }
        Ok(Self::from(BaseElement::from(value)))
    }
}

// FIXME
impl AsBytes for F3G {
    fn as_bytes(&self) -> &[u8] {
        // TODO: take endianness into account
        let self_ptr: *const F3G = self;
        unsafe { slice::from_raw_parts(self_ptr as *const u8, ELEMENT_BYTES) }
    }
}

impl<'a> TryFrom<&'a [u8]> for F3G {
    type Error = DeserializationError;

    /// Converts a slice of bytes into a field element; returns error if the value encoded in bytes
    /// is not a valid field element. The bytes are assumed to encode the element in the canonical
    /// representation in little-endian byte order.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < ELEMENT_BYTES {
            return Err(DeserializationError::InvalidValue(format!(
                "not enough bytes for a full field element; expected {} bytes, but was {} bytes",
                ELEMENT_BYTES,
                bytes.len(),
            )));
        }
        if bytes.len() > ELEMENT_BYTES {
            return Err(DeserializationError::InvalidValue(format!(
                "too many bytes for a field element; expected {} bytes, but was {} bytes",
                ELEMENT_BYTES,
                bytes.len(),
            )));
        }
        let value = bytes
            .try_into()
            .map(u64::from_le_bytes)
            .map_err(|error| DeserializationError::UnknownError(format!("{}", error)))?;
        if value >= M {
            return Err(DeserializationError::InvalidValue(format!(
                "invalid field element: value {} is greater than or equal to the field modulus",
                value
            )));
        }
        Ok(Self::from(BaseElement::from(value)))
    }
}

impl StarkField for F3G {
    /// sage: MODULUS = 2^64 - 2^32 + 1 \
    /// sage: GF(MODULUS).is_prime_field() \
    /// True \
    /// sage: GF(MODULUS).order() \
    /// 18446744069414584321
    const MODULUS: Self::PositiveInteger = M;
    const MODULUS_BITS: u32 = 64;

    /// sage: GF(MODULUS).primitive_element() \
    /// 7
    const GENERATOR: Self = Self::ONE; //Self::from(7)

    /// sage: is_odd((MODULUS - 1) / 2^32) \
    /// True
    const TWO_ADICITY: u32 = 32;

    /// sage: k = (MODULUS - 1) / 2^32 \
    /// sage: GF(MODULUS).primitive_element()^k \
    /// 1753635133440165772
    const TWO_ADIC_ROOT_OF_UNITY: Self = Self::ONE; //Self::from(G);

    fn get_modulus_le_bytes() -> Vec<u8> {
        M.to_le_bytes().to_vec()
    }

    #[inline]
    fn as_int(&self) -> Self::PositiveInteger {
        /*
        if self.dim == 1 {
            self.to_be().as_int()
        } else {
            panic!("Invalid as int: {:?}", *self);
        }
        */
        self.as_elements()[0].as_int()
    }
}

#[cfg(test)]
pub mod tests {
    use std::ops::{Add, Mul};

    use crate::f3g::F3G;
    use winter_math::fields::f64::BaseElement;
    use winter_math::FieldElement;

    #[test]
    fn test_f3g_add() {
        let f1 = F3G::new(
            BaseElement::ONE,
            BaseElement::from(2u32),
            BaseElement::from(3u32),
        );
        let f2 = f1.add(f1);

        let f22 = f1.double();
        assert_eq!(f2, f22);
    }

    #[test]
    fn test_f3g_comparison() {
        let e1 = F3G::new(
            BaseElement::ONE,
            BaseElement::from(2u32),
            BaseElement::from(3u32),
        );

        let elems = e1.as_elements();
        assert_eq!(elems[0], BaseElement::ONE);

        let e11 = F3G::new(
            BaseElement::ONE,
            BaseElement::from(2u32),
            BaseElement::from(3u32),
        );

        let e12 = F3G::new(
            BaseElement::from(2u32),
            BaseElement::from(2u32),
            BaseElement::from(3u32),
        );

        assert_eq!(e1.eq(&e11), true);
        assert_eq!(e1.lt(&e12), true);
    }

    #[test]
    fn test_f3g_exp() {
        let e1 = F3G::new(
            BaseElement::from(5u32),
            BaseElement::from(6u32),
            BaseElement::from(7u32),
        );

        let expected = F3G::new(
            BaseElement::from(9897124412254467696u64),
            BaseElement::from(14730484130337994984u64),
            BaseElement::from(4476495173063158826u64),
        );

        assert_eq!(e1.exp(100).eq(&expected), true);
    }

    #[test]
    fn test_f3g_batch_inverse() {
        let a = F3G::new(
            BaseElement::ONE,
            BaseElement::from(2u32),
            BaseElement::from(3u32),
        );

        let b = a.inv();
        let c = a.mul(b);
        assert_eq!(c.eq(&F3G::ONE3), true);
    }
}
