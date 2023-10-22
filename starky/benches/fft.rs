#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use rayon::prelude::*;
use starky::fft_p::{fft, ifft};
use starky::traits::FieldExtension;
use starky::{f3g::F3G, fft::FFT};
use std::fmt::format;
const MIN_K: usize = 6;
const MAX_K: usize = 24;

fn generate_coeffs<F: FieldExtension>(k: usize) -> Vec<F> {
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

fn bench_standard_fft<F: FieldExtension>(c: &mut Criterion) {
    let mut f = FFT::new();

    let mut group_fft = c.benchmark_group("standard_fft");
    let mut rng = ::rand::thread_rng();
    for k in MIN_K..=MAX_K {
        // prepare data.
        let mut a: Vec<F> = generate_coeffs(k);
        // bench fft
        group_fft.bench_function(format!("fft,k={}", k), |b| {
            b.iter(|| {
                f.fft(&a);
            });
        });
        // bench ifft
        group_fft.bench_function(format!("ifft,k={}", k), |b| {
            b.iter(|| {
                f.ifft(&a);
            });
        });
    }
}

fn bench_p_fft<F: FieldExtension>(c: &mut Criterion) {
    let mut group_fft = c.benchmark_group("p_fft");
    let mut rng = ::rand::thread_rng();
    for k in MIN_K..=MAX_K {
        // prepare data.
        let mut a: Vec<F> = generate_coeffs(k);
        // bench fft
        group_fft.bench_function(format!("p_fft,k={}", k), |b| {
            b.iter(|| {
                fft(&a, 1, k, &mut vec![]);
            });
        });
        // bench ifft
        group_fft.bench_function(format!("p_ifft,k={}", k), |b| {
            b.iter(|| {
                ifft(&a, 1, k, &mut vec![]);
            });
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_standard_fft::<F3G>(c);
    bench_p_fft::<F3G>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
