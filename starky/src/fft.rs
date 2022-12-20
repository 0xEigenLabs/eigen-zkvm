#![allow(dead_code)]
use crate::constant::MG;
use crate::f3g::F3G;
use crate::helper::log2_any;
use winter_math::FieldElement;

pub struct FFT {
    pub roots: Vec<Vec<F3G>>,
}

impl FFT {
    pub fn new() -> Self {
        let s = 32;
        let mut self_ = FFT {
            roots: vec![Vec::new(); s],
        };
        self_.set_roots(core::cmp::min(s, 15));
        self_
    }

    fn set_roots(&mut self, s: usize) {
        let mut i = s;
        while !(i > self.roots.len() && self.roots[i].len() > 0) {
            let mut r = F3G::ONE;
            let nroots = 1 << i;
            self.roots[i] = vec![F3G::ZERO; nroots];
            for j in 0..nroots {
                self.roots[i][j] = r;
                r = r * MG.0[i];
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    pub fn ifft(&mut self, p: &Vec<F3G>) -> Vec<F3G> {
        if p.len() <= 1 {
            return p.clone();
        }

        let bits = log2_any(p.len() - 1) + 1;
        self.set_roots(bits);

        let m = 1 << bits;
        if p.len() != m {
            panic!("Size must be mutiple of 2");
        }
        let res = self._fft(p, bits, 0, 1);
        let twoinvm = F3G::inv(F3G::ONE.mul_scalar(m));
        let mut resn = vec![F3G::ZERO; m];
        for i in 0..m {
            resn[i] = res[(m - i) % m] * twoinvm;
        }
        resn
    }

    pub fn fft(&mut self, p: &Vec<F3G>) -> Vec<F3G> {
        if p.len() <= 1 {
            return p.clone();
        }
        let bits = log2_any(p.len() - 1) + 1;
        self.set_roots(bits);
        let m = 1 << bits;
        if p.len() != m {
            panic!("Size must be mutiple of 2");
        }
        self._fft(p, bits, 0, 1)
    }

    fn _fft(&mut self, p: &Vec<F3G>, bits: usize, offset: usize, step: usize) -> Vec<F3G> {
        let n = 1 << bits;
        if n == 1 {
            return vec![p[offset]];
        } else if n == 2 {
            return vec![p[offset] + p[offset + step], p[offset] - p[offset + step]];
        }

        let ndiv2 = n >> 1;
        let p1 = self._fft(p, bits - 1, offset, step * 2);
        let p2 = self._fft(p, bits - 1, offset + step, step * 2);

        let mut out = vec![F3G::ZERO; n];

        for i in 0..ndiv2 {
            out[i] = p1[i] + p2[i] * self.roots[bits][i];
            out[i + ndiv2] = p1[i] - p2[i] * self.roots[bits][i];
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use crate::f3g::F3G;
    use crate::fft::FFT;
    #[test]
    fn test_single_fft() {
        let mut f = FFT::new();
        let a: Vec<F3G> = vec![1u32, 2u32, 3u32, 5u32]
            .iter()
            .map(|e| F3G::from(*e))
            .collect();
        let aa = f.fft(&a);
        let ac = f.ifft(&aa);
        for i in 0..a.len() {
            assert_eq!(a[i], ac[i]);
        }
    }

    #[test]
    fn test_random_fft() {
        let mut f = FFT::new();
        let mut a: Vec<F3G> = Vec::new();
        for _i in 0..64 {
            a.push(F3G::random());
        }
        let aa = f.fft(&a);
        let ac = f.ifft(&aa);
        for i in 0..a.len() {
            assert_eq!(a[i], ac[i]);
        }
    }
}
