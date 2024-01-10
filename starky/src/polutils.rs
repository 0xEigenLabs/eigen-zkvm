use crate::constant::SHIFT;
use crate::fft::FFT;
use crate::traits::FieldExtension;

pub fn pol_mul_axi<F: FieldExtension>(p: &mut [F], init: F, acc: &F) {
    let mut r = init;
    for pi in p {
        *pi *= r;
        r *= *acc;
    }
}

pub fn eval_pol<F: FieldExtension>(p: &[F], x: &F) -> F {
    if p.is_empty() {
        return F::ZERO;
    }
    let mut res = p[p.len() - 1];
    for i in (0..(p.len() - 1)).rev() {
        res = res * *x + p[i];
    }
    res
}

#[allow(dead_code)]
pub fn extend_pol<F: FieldExtension>(p: &[F], extend_bits: usize) -> Vec<F> {
    let mut standard_fft = FFT::new();
    let mut res = standard_fft.ifft(p);
    pol_mul_axi(&mut res, F::ONE, &F::from(*SHIFT));
    let n_extend = (p.len() << extend_bits) - p.len();
    let zeros = vec![F::ZERO; n_extend];
    res.extend_from_slice(&zeros);
    standard_fft.fft(&res)
}

pub fn batch_inverse<F: FieldExtension>(elems: &[F]) -> Vec<F> {
    if elems.is_empty() {
        return vec![];
    }

    let mut tmp: Vec<F> = vec![F::ZERO; elems.len()];
    tmp[0] = elems[0];
    for i in 1..elems.len() {
        tmp[i] = elems[i] * (tmp[i - 1]);
    }
    let mut z = tmp[tmp.len() - 1].inv();
    let mut res: Vec<F> = vec![F::ZERO; elems.len()];
    for i in (1..elems.len()).rev() {
        res[i] = z * tmp[i - 1];
        z *= elems[i];
    }
    res[0] = z;
    res
}
