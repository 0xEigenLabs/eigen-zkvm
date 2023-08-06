use crate::scalar_gl::Fr;
use crate::ff::*;
use core::ops::{Add, Mul, Neg, Sub};
use rand::Rand;

impl Add for Fr {
    type Output = Fr;

    fn add(self, other: Fr) -> Fr {
        let mut result = self;
        result.add_assign(&other);
        result
    }
}

impl Sub for Fr {
    type Output = Fr;

    fn sub(self, other: Fr) -> Fr {
        let mut result = self;
        result.sub_assign(&other); // note the reference
        result
    }
}

impl Mul for Fr {
    type Output = Fr;

    fn mul(self, other: Fr) -> Fr {
        let mut result = self;
        result.mul_assign(&other);
        result
    }
}

impl Neg for Fr {
    type Output = Fr;

    fn neg(self) -> Self::Output {
        let mut result = self;
        result.negate();
        result
    }
}

pub(crate) fn test_add_neg_sub_mul() {
    let mut rng = rand::thread_rng();
    let x = Fr::rand(&mut rng);
    let y = Fr::rand(&mut rng);
    let z = Fr::rand(&mut rng);
    let mut x_clone = x.clone();
    x_clone.square();
    let x_squared = x_clone;
    assert_eq!(x + (-x), Fr::zero());
    assert_eq!(-x, Fr::zero() - x);
    assert_eq!(x, x * Fr::one());
    assert_eq!(x * (-x), -x_squared);
    assert_eq!(x + y, y + x);
    assert_eq!(x * y, y * x);
    assert_eq!(x * (y * z), (x * y) * z);
    assert_eq!(x - (y + z), (x - y) - z);
    assert_eq!((x + y) - z, x + (y - z));
    assert_eq!(x * (y + z), x * y + x * z);
}

pub(crate) fn test_inv() {
    let mut rng = rand::thread_rng();
    let x = Fr::rand(&mut rng);
    let y = Fr::rand(&mut rng);
    let z = Fr::rand(&mut rng);
    let mut x_clone = x.clone();
    x_clone.inverse();
    let x_inversed = x_clone;
    assert_eq!(x * x_inversed, Fr::one());
    assert_eq!(x_inversed * x, Fr::one());
    // x_clone.square();
    // let x_inversed_squared = x_clone;
    // let mut x_clone1 = x.clone();
    // x_clone1.square();
    // x_clone1.inverse();
    // let x_squared_inversed = x_clone1;
    // assert_eq!(x_squared_inversed, x_inversed_squared);
}


#[cfg(test)]
mod tests {
    use super::test_add_neg_sub_mul;
    use super::test_inv;

    #[test]
    #[allow(clippy::eq_op)]
    fn check_add_neg_sub_mul() {
        test_add_neg_sub_mul();
    }
    #[test]
    #[allow(clippy::eq_op)]
    fn check_inv() {
        test_inv();
    }
}