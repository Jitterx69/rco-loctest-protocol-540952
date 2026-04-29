use criterion::{criterion_group, criterion_main, Criterion};
use rco_enclave::oracle::{DecentralizedJacobianOracle};

fn bench_root_consistency(c: &mut Criterion) {
    let mut oracle = DecentralizedJacobianOracle::new(1000);
    let root_hash = [0u8; 32];
    
    // Simulate multi-cluster anchors
    for i in 0..10 {
        oracle.update_root_anchor(i as u64, root_hash);
    }

    c.bench_function("SOVEREIGN-ROOT-CONSISTENCY/verification", |b| {
        b.iter(|| {
            let consistent = oracle.verify_root_consistency(&root_hash);
            assert!(consistent);
        })
    });
}

criterion_group!(benches, bench_root_consistency);
criterion_main!(benches);
