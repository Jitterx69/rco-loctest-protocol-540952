use criterion::{criterion_group, criterion_main, Criterion};
use rco_enclave::synthesis::HardwareBoundSynthesis;

fn bench_planetary_synthesis(c: &mut Criterion) {
    let synthesis = HardwareBoundSynthesis::new();
    // Simulate 10 cluster roots
    let roots = vec![[0u8; 32]; 10];

    c.bench_function("PLANETARY-SYNTHESIS/multi_cluster_root_fusion", |b| {
        b.iter(|| {
            let global_root = synthesis.synthesize_multi_cluster(&roots);
            assert_eq!(global_root.len(), 32);
        })
    });
}

criterion_group!(benches, bench_planetary_synthesis);
criterion_main!(benches);
