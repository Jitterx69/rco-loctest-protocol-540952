use criterion::{criterion_group, criterion_main, Criterion};
use rco_enclave::routing::ManifoldRoutingLogic;

fn bench_self_repair(c: &mut Criterion) {
    let mut routing = ManifoldRoutingLogic::new();
    // Simulate 100 failed shards
    let failed_ids: Vec<u64> = (0..100).collect();

    c.bench_function("SELF-REPAIR/simplicial_healing_100_shards", |b| {
        b.iter(|| {
            let holes = routing.detect_holes();
            routing.heal_topology(&failed_ids);
        })
    });
}

criterion_group!(benches, bench_self_repair);
criterion_main!(benches);
