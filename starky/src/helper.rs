use std::ops::Mul;
///exports.getKs = function getKs(Fr, n) {
///    const ks = [Fr.k];
///    for (let i=1; i<n; i++) {
///        ks[i] = Fr.mul(ks[i-1], ks[0]);
///    }
///    return ks;
///}
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;

pub fn get_ks(fr: &BaseElement, n: usize) -> Vec<BaseElement> {
    let mut ks: Vec<BaseElement> = vec![BaseElement::ZERO; n];
    ks[0] = BaseElement::from(12275445934081160404u64);
    for i in 1..n {
        ks[i] = ks[i - 1].mul(ks[0])
    }
    ks
}
