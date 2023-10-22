#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use starky::fft_p::{fft, ifft};
use starky::traits::FieldExtension;
use starky::{f3g::F3G, fft::FFT};

const MIN_K: usize = 6;
const MAX_K: usize = 24;

fn bench_standard_fft<F: FieldExtension>(c: &mut Criterion) {
    let mut f = FFT::new();

    let mut group_fft = c.benchmark_group("fft");
    let mut group_ifft = c.benchmark_group("ifft");
    let mut rng = ::rand::thread_rng();
    for k in MIN_K..=MAX_K {
        // prepare data.
        let mut a = (0..(1 << k))
            .map(|_| <F3G as rand::Rand>::rand(&mut rng))
            .collect::<Vec<_>>();
        // bench fft
        group_fft.bench_function(BenchmarkId::new("k", k), |b| {
            b.iter(|| {
                f.fft(&a);
            });
        });
        // bench ifft
        group_ifft.bench_function(BenchmarkId::new("k", k), |b| {
            b.iter(|| {
                f.ifft(&a);
            });
        });
    }
}

fn bench_p_fft<F: FieldExtension>(c: &mut Criterion) {
    let mut group_fft = c.benchmark_group("p_fft");
    let mut group_ifft = c.benchmark_group("p_ifft");
    let mut rng = ::rand::thread_rng();
    for k in MIN_K..=MAX_K {
        // prepare data.
        let mut a = (0..(1 << k))
            .map(|_| <F3G as rand::Rand>::rand(&mut rng))
            .collect::<Vec<_>>();
        // bench fft
        group_fft.bench_function(BenchmarkId::new("k", k), |b| {
            b.iter(|| {
                fft(&a, 1, k, &mut vec![]);
            });
        });
        // bench ifft
        group_ifft.bench_function(BenchmarkId::new("k", k), |b| {
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
