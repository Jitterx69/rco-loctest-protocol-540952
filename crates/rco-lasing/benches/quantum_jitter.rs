use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::controller::LasingController;
use nalgebra::DVector;

fn bench_quantum_jitter(c: &mut Criterion) {
    let dim = 1000;
    let mut controller = LasingController::new(dim, 10, 0.5, 1.0);
    
    // Simulate Sub-Lambda stabilization (1.0K)
    controller.quantum.update_jitter_floor(1.0);
    
    let epsilon = DVector::from_element(10, 0.1);
    let jacobian = nalgebra::DMatrix::from_element(10, dim, 0.01);

    c.bench_function("QUANTUM-BOUND-JITTER/heisenberg_gain_adjustment", |b| {
        b.iter(|| {
            // Apply quantum-bound gain logic
            let heisenberg_gain = controller.quantum.compute_heisenberg_gain(1.0, epsilon.norm());
            assert!(heisenberg_gain > 0.0 && heisenberg_gain < 1.0);
        })
    });
}

criterion_group!(benches, bench_quantum_jitter);
criterion_main!(benches);
