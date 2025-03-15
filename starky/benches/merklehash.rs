#![allow(clippy::needless_range_loop)]
use criterion::*;
use fields::field_gl::Fr as FGL;
use rayon::prelude::*;
use starky::merklehash::MerkleTreeGL;
use starky::traits::MerkleTree;
mod perf;

fn run_merklehash(pols: Vec<FGL>) {
    let n = 1 << 24;
    let idx = 32;
    let n_pols = 10;

    let now = std::time::Instant::now();
    let mut tree: MerkleTreeGL = MerkleTree::new();
    tree.merkelize(pols, n_pols, n).unwrap();
    log::trace!("time cost: {}", now.elapsed().as_secs());
    let (group_elements, mp) = tree.get_group_proof(idx).unwrap();
    let root = tree.root();
    assert!(tree.verify_group_proof(&root, &mp, idx, &group_elements).unwrap());
}

fn merklehash_group_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("merklehash");

    let n = 1 << 24;
    let n_pols = 10;
    let mut pols: Vec<FGL> = vec![FGL::ZERO; n_pols * n];

    rayon::scope(|_s| {
        pols.par_chunks_mut(n).enumerate().for_each(|(i, bb)| {
            for j in 0..n {
                bb[j] = FGL::from((j + i * n) as u64)
            }
        });
    });
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    {
        let data = &(&pols);
        group.bench_with_input(
            BenchmarkId::new("merkelize", format!("height*n_pols={}", pols.len())),
            data,
            |b, s| {
                b.iter(|| run_merklehash(s.to_vec()));
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = merklehash_group_bench
}
criterion_main!(benches);
