/// A test/bench tools
use crate::traits::FieldExtension;
use ff::PrimeField;
use rayon::prelude::*;

// concurrency generate random goldfields. with specific k.
pub fn gen_rand_goldfields<F: FieldExtension>(k: usize) -> Vec<F> {
    let n = 1 << k;
    let mut parts = vec![F::zero(); n];

    parts.par_iter_mut().for_each(|p| {
        let mut rng = ::rand::thread_rng();
        *p = <F as rand::Rand>::rand(&mut rng)
    });
    parts
}

// concurrency generate random fields. with specific k.
pub fn gen_rand_fields<F: PrimeField>(k: usize) -> Vec<F> {
    let n = 1 << k;
    let mut parts = vec![F::zero(); n];

    parts.par_iter_mut().for_each(|p| {
        let mut rng = ::rand::thread_rng();

        *p = <F as rand::Rand>::rand(&mut rng)
    });
    parts
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::f3g::F3G;
    use crate::field_bn128::Fr;

    #[test]
    fn test_gen_rand_goldfields() {
        let k: usize = 1;
        let n = 1 << k;
        let res = gen_rand_goldfields::<F3G>(k);
        assert_eq!(n, res.len());
    }

    #[test]
    fn test_gen_rand_fields() {
        let k: usize = 1;
        let n = 1 << k;
        let res = gen_rand_fields::<Fr>(k);
        assert_eq!(n, res.len());
    }
}
