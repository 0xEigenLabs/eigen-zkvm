use crate::constant::SHIFT;
use crate::fft::FFT;
use crate::traits::FieldExtension;

pub fn pol_mul_axi<F: FieldExtension>(p: &mut Vec<F>, init: F, acc: &F) {
    let mut r = init;
    for i in 0..p.len() {
        p[i] *= r;
        r *= *acc;
    }
}

pub fn eval_pol<F: FieldExtension>(p: &Vec<F>, x: &F) -> F {
    if p.len() == 0 {
        return F::ZERO;
    }
    let mut res = p[p.len() - 1];
    for i in (0..(p.len() - 1)).rev() {
        res = res * *x + p[i];
    }
    res
}

#[allow(dead_code)]
pub fn extend_pol<F: FieldExtension>(p: &Vec<F>, extend_bits: usize) -> Vec<F> {
    let mut standard_fft = FFT::new();
    let mut res = standard_fft.ifft(&p);
    pol_mul_axi(&mut res, F::ONE, &F::from(SHIFT.clone()));
    let n_extend = (p.len() << extend_bits) - p.len();
    let zeros = vec![F::ZERO; n_extend];
    res.extend_from_slice(&zeros);
    standard_fft.fft(&res)
}
