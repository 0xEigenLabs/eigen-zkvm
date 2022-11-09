use rand_utils::rand_vector;
use winter_math::fields::f64::BaseElement;
use winter_math::fields::CubeExtension;
use winter_math::FieldElement;

use core::ops::{Add, Div, Mul, Neg, Sub};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct F3G(CubeExtension<BaseElement>);

impl F3G {
    pub fn new(a: BaseElement, b: BaseElement, c: BaseElement) -> Self {
        F3G(CubeExtension::new(a, b, c))
    }

    pub fn zero() -> Self {
        Self(CubeExtension::<BaseElement>::ZERO)
    }

    pub fn add(self, added: Self) -> Self {
        return Self(self.0.add(added.0));
    }

    pub fn sub(self, minuend: Self) -> Self {
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

    pub fn mul(self, rhs: Self) -> Self {
        Self(self.0.mul(rhs.0))
    }

    pub fn mulScalar(self, b: BaseElement) -> Self {
        let s = Self::new(b, b, b);
        self.mul(s)
    }

    pub fn square(self) -> Self {
        Self(self.0.square())
    }

    pub fn div(self, rhs: Self) -> Self {
        Self(self.0.div(rhs.0))
    }

    pub fn eq(self, rhs: Self) -> bool {
        self == rhs
    }

    pub fn gt(self, rhs: Self) -> bool {
        self.0 .0 > rhs.0 .0
    }
    pub fn geq(self, rhs: Self) -> bool {
        self.eq(rhs) || self.gt(rhs)
    }

    pub fn lt(self, rhs: Self) -> bool {
        !self.geq(rhs)
    }

    pub fn leq(self, rhs: Self) -> bool {
        !self.gt(rhs)
    }

    pub fn is_zero(self) -> bool {
        self.eq(Self::zero())
    }

    pub fn e() {
        unreachable!("f3g::e");
    }

    pub fn random() -> Self {
        let cube = rand_vector::<BaseElement>(3);
        Self::new(cube[0], cube[1], cube[2])
    }

    pub fn exp(self, e: BaseElement) -> Self {
        /*
        if e == BaseElement::ZERO {
            return Self::one();
        }
        let bits = e.bits();
        */
        unreachable!("111");
        Self::zero()
    }

    pub fn pow(self, e: BaseElement) -> Self {
        self.exp(e)
    }

    pub fn batch_inverse(elems: &[Self]) -> Vec<Self> {
        if elems.len() == 0 {
            return vec![];
        }

        let mut tmp: Vec<Self> = vec![Self::zero(); elems.len()];
        tmp[0] = elems[0];
        for i in 1..elems.len() {
            tmp[i] = elems[i].mul(tmp[i - 1]);
        }
        let z = tmp[tmp.len() - 1].inv();
        let mut res: Vec<Self> = vec![Self::zero(); elems.len()];
        for i in (1..elems.len()).rev() {
            res[i] = z.mul(tmp[i - 1]);
            z = z.mul(elems[i]);
        }
        res[0] = z;
        res
    }

    pub fn from_rpr_le() {
        unreachable!("f3g::from_rpr_le");
    }
}

#[cfg(test)]
pub mod tests {
    use crate::f3g::F3G;
    use winter_math::fields::f64::BaseElement;
    use winter_math::FieldElement;

    #[test]
    fn test_f3g_add() {
        let f1 = F3G::new(
            BaseElement::from(1u32),
            BaseElement::from(2u32),
            BaseElement::from(3u32),
        );
        let f2 = f1.add(f1);

        let f22 = f1.double();
        assert_eq!(f2, f22);
    }
}
