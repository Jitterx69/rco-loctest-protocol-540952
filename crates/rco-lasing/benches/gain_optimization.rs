//! GAIN-OPTIMIZATION Benchmark
//!
//! Evaluates the convergence time of the PID-Reflexive Loop (Target: < 12 iterations).

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use nalgebra::{DMatrix, DVector};
use rco_lasing::controller::LasingController;

fn bench_gain_convergence(c: &mut Criterion) {
    let mut group = c.benchmark_group("GAIN-OPTIMIZATION");
    
    group.bench_function("pid_reflexive_step", |b| {
        let mut controller = LasingController::new(100, 10, 0.338, 1.0);
        let epsilon = DVector::from_element(10, 0.05);
        let jacobian = DMatrix::from_element(10, 100, 0.1);

        b.iter(|| {
            black_box(controller.compute_lasing_force(&epsilon, &jacobian));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_gain_convergence);
criterion_main!(benches);
