#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use starky::dev::gen_rand_goldfields;
use starky::f3g::F3G;
use starky::polutils::batch_inverse;
use starky::traits::FieldExtension;

const MIN_K: usize = 6;
const MAX_K: usize = 24;

fn bench_batch_inverse<F: FieldExtension>(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_inverse");
    for k in MIN_K..=MAX_K {
        let a: Vec<F> = gen_rand_goldfields(k);
        group.bench_function(BenchmarkId::new("k", k), |b| {
            b.iter(|| batch_inverse(&a));
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_batch_inverse::<F3G>(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
