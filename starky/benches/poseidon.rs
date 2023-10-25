#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use ff::Field;
use starky::dev::gen_rand_fields;
use starky::field_bn128::Fr;
use starky::poseidon_bn128_opt::Poseidon;
const MIN_K: usize = 6;
const MAX_K: usize = 24;

fn bench_poseidon128(c: &mut Criterion) {
    let poseidon = Poseidon::new();
    let init = Fr::zero();

    let mut group = c.benchmark_group("poseidon128_opt");
    for k in MIN_K..=MAX_K {
        let a: Vec<Fr> = gen_rand_fields(k);
        group.bench_function(BenchmarkId::new("k", k), |b| {
            b.iter(|| poseidon.hash(&a, &init));
        });
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_poseidon128(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
