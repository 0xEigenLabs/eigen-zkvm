use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use rayon::prelude::*;
use starky::f3g::F3G;
use starky::merklehash_bn128::MerkleTree;
use winter_math::{FieldElement, StarkField};

fn run_merklehash(pols: Vec<F3G>) {
    let N = 1 << 24;
    let idx = 32;
    let nPols = 600;

    /*
    let mut pols: Vec<F3G> = vec![F3G::ZERO; nPols * N];
    for i in 0..N {
        for j in 0..nPols {
            pols[i * nPols + j] = F3G::from((i + j * 1000));
        }
    }
    rayon::scope(|s| {
        pols.par_chunks_mut(N).enumerate().for_each(|(i, bb)| {
            for j in 0..N {
                bb[j] = F3G::from((i + j * 1000))
            }
        });
    });
    */

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

    let N = 1 << 24;
    let nPols = 600;
    let mut pols: Vec<F3G> = vec![F3G::ZERO; nPols * N];

    rayon::scope(|s| {
        pols.par_chunks_mut(N).enumerate().for_each(|(i, bb)| {
            for j in 0..N {
                bb[j] = F3G::from(j + i * N)
            }
        });
    });
    group.sample_size(10);
    for data in [pols].iter() {
        group.bench_with_input(
            BenchmarkId::new("merkelize", format!("height=2^{}, n_pols={}", 24, 600)),
            data,
            |b, s| {
                b.iter(|| run_merklehash(s.to_vec()));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, merklehash_group_bench);
criterion_main!(benches);
