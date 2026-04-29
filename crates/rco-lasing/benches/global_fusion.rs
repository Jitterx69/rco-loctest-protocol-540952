use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::fusion::LorentzInvariantFusion;
use nalgebra::DVector;

fn bench_global_fusion(c: &mut Criterion) {
    let dim = 1000;
    let fusion = LorentzInvariantFusion::new();
    let num_shards = 10;
    
    let states: Vec<DVector<f64>> = (0..num_shards).map(|_| DVector::from_element(dim, 0.1)).collect();
    let entropies = vec![0.01; num_shards];
    let velocities = vec![0.5; num_shards];

    c.bench_function("GLOBAL-FUSION/state_entanglement_10_shards", |b| {
        b.iter(|| {
            let fused = fusion.fuse_states(states.clone(), &entropies, &velocities);
            assert_eq!(fused.len(), dim);
        })
    });
}

criterion_group!(benches, bench_global_fusion);
criterion_main!(benches);
