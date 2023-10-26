#[macro_use]
extern crate criterion;

use criterion::Criterion;
use starky::dev::gen_rand_goldfields;
use starky::fft_p::{fft, ifft};
use starky::traits::FieldExtension;
use starky::{f3g::F3G, fft::FFT};

const MIN_K: usize = 6;
const MAX_K: usize = 24;

fn bench_standard_fft<F: FieldExtension>(c: &mut Criterion) {
    let mut f = FFT::new();

    let mut group = c.benchmark_group("standard_fft");

    for k in MIN_K..=MAX_K {
        let a: Vec<F> = gen_rand_goldfields(k);

        group.bench_function(format!("fft/k/{}", k), |b| {
            b.iter(|| {
                f.fft(&a);
            });
        });

        group.bench_function(format!("ifft/k/{}", k), |b| {
            b.iter(|| {
                f.ifft(&a);
            });
        });
    }
}

fn bench_p_fft<F: FieldExtension>(c: &mut Criterion) {
    let mut group = c.benchmark_group("p_fft");
    for k in MIN_K..=MAX_K {
        // prepare data.
        let a: Vec<F> = gen_rand_goldfields(k);
        // bench fft
        group.bench_function(format!("p_fft/k/{}", k), |b| {
            b.iter(|| {
                fft(&a, 1, k, &mut vec![]);
            });
        });
        // bench ifft
        group.bench_function(format!("p_ifft/k/{}", k), |b| {
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
