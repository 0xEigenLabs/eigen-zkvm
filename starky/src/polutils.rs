use crate::constant::SHIFT;
use crate::f3g::F3G;
use crate::fft::FFT;
use winter_math::FieldElement;

pub fn pol_mul_axi(p: &mut Vec<F3G>, init: F3G, acc: &F3G) {
    let mut r = init;
    for i in 0..p.len() {
        p[i] *= r;
        r *= *acc;
    }
}

pub fn eval_pol(p: &Vec<F3G>, x: &F3G) -> F3G {
    if p.len() == 0 {
        return F3G::ZERO;
    }
    let mut res = p[p.len() - 1];
    for i in (0..(p.len() - 1)).rev() {
        res = res * *x + p[i];
    }
    res
}

#[allow(dead_code)]
pub fn extend_pol(p: &Vec<F3G>, extend_bits: usize) -> Vec<F3G> {
    log::debug!("res");
    crate::helper::pretty_print_array(p);
    let mut standard_fft = FFT::new();
    let mut res = standard_fft.ifft(&p);
    log::debug!("ifft");
    crate::helper::pretty_print_array(&res);
    pol_mul_axi(&mut res, F3G::ONE, &SHIFT);
    log::debug!("pol_mul_axi");
    crate::helper::pretty_print_array(&res);
    let n_extend = (p.len() << extend_bits) - p.len();
    let zeros = vec![F3G::ZERO; n_extend];
    res.extend_from_slice(&zeros);
    let res = standard_fft.fft(&res);
    log::debug!("fft");
    crate::helper::pretty_print_array(&res);
    res
}
