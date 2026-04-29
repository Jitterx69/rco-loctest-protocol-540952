use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::recursive::{HyperRecursiveFinality, HyperProof};

fn bench_hyper_finality(c: &mut Criterion) {
    let mut recursive = HyperRecursiveFinality::new();
    let proofs: Vec<HyperProof> = (0..100).map(|i| HyperProof {
        root_hash: [i as u8; 32],
        proof_depth: 10000,
    }).collect();

    c.bench_function("HYPER-FINALITY/recursive_aggregation_1M_depth", |b| {
        b.iter(|| {
            let root = recursive.aggregate_proofs(proofs.clone());
            assert_ne!(root, [0u8; 32]);
        })
    });
}

criterion_group!(benches, bench_hyper_finality);
criterion_main!(benches);
