#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use starky::dev::gen_rand_fields;
use starky::traits::{batch_inverse, FieldExtension};

const MIN_K: usize = 6;
const MAX_K: usize = 24;

fn criterion_benchmark<F: FieldExtension>(c: &mut Criterion) {
    let mut group_fft = c.benchmark_group("batch_inverse");
    for k in MIN_K..=MAX_K {
        let a: Vec<F> = gen_rand_fields(k);
        group_fft.bench_function(BenchmarkId::new("k", k), |b| {
            b.iter(|| batch_inverse(&a));
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
