use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::evolution::AutonomousManifoldEvolution;
use nalgebra::DVector;

fn bench_manifold_evolution(c: &mut Criterion) {
    let mut evolution = AutonomousManifoldEvolution::new();
    let mut state = DVector::from_element(1000, 1.0);

    c.bench_function("MANIFOLD-EVOLUTION/topology_optimization", |b| {
        b.iter(|| {
            evolution.evaluate_fitness(0.1);
            evolution.re_triangulate(&mut state);
            evolution.adapt_gain(1.0);
        })
    });
}

criterion_group!(benches, bench_manifold_evolution);
criterion_main!(benches);
