use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use starky::f3g::F3G;
use starky::merklehash_bn128::MerkleTree;
use winter_math::{FieldElement, StarkField};

fn run_merklehash(size: &(usize, usize)) {
    let N = 1 << size.0;
    let idx = 32;
    let nPols = size.1;
    let mut pols: Vec<F3G> = vec![F3G::ZERO; nPols * N];
    for i in 0..N {
        for j in 0..nPols {
            pols[i * nPols + j] = F3G::from((i + j * 1000) as u32);
        }
    }

    let tree = MerkleTree::merkelize(pols, nPols, N).unwrap();
    let (groupElements, mp) = tree.get_group_proof(idx).unwrap();
    let root = tree.root();
    assert_eq!(
        tree.verify_group_proof(&root, &mp, idx, &groupElements)
            .unwrap(),
        true
    );
}

fn merklehash_group_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("merklehash");
    group.sample_size(10);
    for data in [(18, 10), (20, 10), (24, 10), (24, 100), (24, 600)].iter() {
        group.bench_with_input(
            BenchmarkId::new(
                "merkelize",
                format!("height=2^{}, n_pols={}", data.0, data.1),
            ),
            &data,
            |b, &s| {
                b.iter(|| run_merklehash(s));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, merklehash_group_bench);
criterion_main!(benches);
