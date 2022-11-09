#![allow(dead_code)]
use core::slice;
use rand_utils::rand_vector;
use winter_math::fields::f64::BaseElement;
use winter_math::fields::CubeExtension;
use winter_math::{FieldElement, StarkField};
use winter_utils::AsBytes;

use core::ops::{Add, Div, Mul, Neg, Sub};

/// GF(2^3) implementation
/// Prime: 0xFFFFFFFF00000001
/// Irreducible polynomial: x^3 - x -1

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct F3G(CubeExtension<BaseElement>);

impl F3G {
    pub fn new(a: BaseElement, b: BaseElement, c: BaseElement) -> Self {
        F3G(CubeExtension::<BaseElement>::new(a, b, c))
    }

    pub fn zero() -> Self {
        Self(CubeExtension::<BaseElement>::ZERO)
    }

    pub fn one() -> Self {
        Self(CubeExtension::<BaseElement>::ONE)
    }

    fn as_base_elements(&self) -> Vec<BaseElement> {
        let cc = &[self.0];
        CubeExtension::<BaseElement>::as_base_elements(cc).to_vec()
    }

    pub fn add(self, added: &Self) -> Self {
        return Self(self.0.add(added.0));
    }

    pub fn sub(self, minuend: &Self) -> Self {
        Self(self.0.add(minuend.0))
    }

    pub fn double(self) -> Self {
        Self(self.0.double())
    }

    pub fn neg(self) -> Self {
        Self(self.0.neg())
    }
    pub fn inv(self) -> Self {
        Self(self.0.inv())
    }

    pub fn mul(self, rhs: &Self) -> Self {
        Self(self.0.mul(rhs.0))
    }

    pub fn mul_scalar(self, b: BaseElement) -> Self {
        let s = Self::new(b, b, b);
        self.mul(&s)
    }

    pub fn square(self) -> Self {
        Self(self.0.square())
    }

    pub fn div(self, rhs: &Self) -> Self {
        Self(self.0.div(rhs.0))
    }

    pub fn eq(self, rhs: &Self) -> bool {
        self == *rhs
    }

    pub fn gt(self, rhs: &Self) -> bool {
        let les = self.as_base_elements();
        let res = rhs.as_base_elements();
        (les[0].as_int() > res[0].as_int())
            || ((les[0].as_int() == res[0].as_int()) && (les[1].as_int() == res[1].as_int()))
            || ((les[0].as_int() == res[0].as_int())
                && (les[1].as_int() == res[1].as_int())
                && (les[2].as_int() > res[2].as_int()))
    }

    pub fn geq(self, rhs: &Self) -> bool {
        self.eq(rhs) || self.gt(rhs)
    }

    pub fn lt(self, rhs: &Self) -> bool {
        !self.geq(rhs)
    }

    pub fn leq(self, rhs: &Self) -> bool {
        !self.gt(rhs)
    }

    pub fn is_zero(self) -> bool {
        self.eq(&Self::zero())
    }

    pub fn random() -> Self {
        let cube = rand_vector::<BaseElement>(3);
        Self::new(cube[0], cube[1], cube[2])
    }

    pub fn exp(self, e_: u64) -> Self {
        let mut e = e_;
        if e == 0 {
            return Self(CubeExtension::ONE);
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
            return Self(CubeExtension::ONE);
        }

        let mut res = self;
        for i in (0..bits.len() - 1).rev() {
            res = res.square();
            if bits[i] == 1 {
                res = res.mul(&self);
            }
        }
        res
    }

    pub fn pow(self, e: u64) -> Self {
        self.exp(e)
    }

    pub fn batch_inverse(elems: &[Self]) -> Vec<Self> {
        if elems.len() == 0 {
            return vec![];
        }

        let mut tmp: Vec<Self> = vec![Self::zero(); elems.len()];
        tmp[0] = elems[0];
        for i in 1..elems.len() {
            tmp[i] = elems[i].mul(&tmp[i - 1]);
        }
        let mut z = tmp[tmp.len() - 1].inv();
        let mut res: Vec<Self> = vec![Self::zero(); elems.len()];
        for i in (1..elems.len()).rev() {
            res[i] = z.mul(&tmp[i - 1]);
            z = z.mul(&elems[i]);
        }
        res[0] = z;
        res
    }
}

#[cfg(test)]
pub mod tests {
    use crate::f3g::F3G;
    use winter_math::fields::f64::BaseElement;
    use winter_math::fields::CubeExtension;
    use winter_math::FieldElement;
    use winter_math::StarkField;

    #[test]
    fn test_f3g_add() {
        let f1 = F3G::new(
            BaseElement::from(1u32),
            BaseElement::from(2u32),
            BaseElement::from(3u32),
        );
        let f2 = f1.add(&f1);

        let f22 = f1.double();
        assert_eq!(f2, f22);
    }

    #[test]
    fn test_f3g_comparison() {
        let e1 = F3G::new(
            BaseElement::from(1u32),
            BaseElement::from(2u32),
            BaseElement::from(3u32),
        );

        let elems = e1.as_base_elements();
        assert_eq!(elems[0], BaseElement::from(1u32));

        let e11 = F3G::new(
            BaseElement::from(1u32),
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
            BaseElement::from(1u32),
            BaseElement::from(2u32),
            BaseElement::from(3u32),
        );

        let b = a.inv();
        let c = a.mul(&b);
        assert_eq!(c.eq(&F3G::one()), true);
    }
}
