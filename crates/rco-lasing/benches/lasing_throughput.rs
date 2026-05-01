//! LASING-THROUGHPUT Benchmark
//!
//! Evaluates manifold coherence stability under high-velocity ingestion (Target: 5M steps/sec).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use nalgebra::{DMatrix, DVector};
use rco_lasing::controller::LasingController;

fn bench_lasing_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("LASING-THROUGHPUT");

    for load in [1000, 5000, 10000].iter() {
        group.bench_with_input(format!("load_{}_steps", load), load, |b, &load| {
            let param_dim = 100;
            let obs_dim = 10;
            let mut controller = LasingController::new(param_dim, obs_dim, 0.5, 1.0);
            let epsilon = DVector::from_element(obs_dim, 0.01);
            let jacobian = DMatrix::from_element(obs_dim, param_dim, 0.1);

            b.iter(|| {
                for _ in 0..load {
                    let _force = black_box(controller.compute_lasing_force(&epsilon, &jacobian));
                }
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_lasing_throughput);
criterion_main!(benches);
