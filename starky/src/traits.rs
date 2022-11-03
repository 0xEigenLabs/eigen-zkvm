use crate::poseidon_bn128::Fr;
use winter_math::fields::f64::BaseElement;

pub trait FieldMapping {
    fn to_GL(f: &Fr) -> [BaseElement; 4];
    fn to_montgomery(e: &Fr) -> Fr;
    fn to_BN128(b: &[BaseElement; 4]) -> Fr;
}
