/// A test/bench tools
use crate::traits::{batch_inverse, FieldExtension};
use ff::PrimeField;

// concurrency generate random goldfields. with specific k.
pub fn gen_rand_goldfields<F: FieldExtension>(k: usize) -> Vec<F> {
    let num_threads = rayon::current_num_threads();
    let mut parts = vec![F::one(); 1 << k];
    rayon::scope(|scope| {
        for out in parts.chunks_mut(num_threads) {
            scope.spawn(move |_| {
                let mut rng = ::rand::thread_rng();
                for i in 0..num_threads {
                    out[i] = <F as rand::Rand>::rand(&mut rng)
                }
            })
        }
    });
    parts
}

// concurrency generate random fields. with specific k.
pub fn gen_rand_fields<F: PrimeField>(k: usize) -> Vec<F> {
    let num_threads = rayon::current_num_threads();
    let mut parts = vec![F::one(); 1 << k];
    rayon::scope(|scope| {
        for out in parts.chunks_mut(num_threads) {
            scope.spawn(move |_| {
                let mut rng = ::rand::thread_rng();
                for i in 0..num_threads {
                    out[i] = <F as rand::Rand>::rand(&mut rng)
                }
            })
        }
    });
    parts
}
