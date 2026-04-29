use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::controller::LasingController;
use nalgebra::DVector;

fn bench_loop_stability(c: &mut Criterion) {
    let mut controller = LasingController::new(100, 10, 0.5, 1.0);
    let epsilon = DVector::from_element(10, 0.1);
    let jacobian = nalgebra::DMatrix::from_element(10, 100, 0.01);

    c.bench_function("LOOP-STABILITY/rfc_sync_step", |b| {
        b.iter(|| {
            controller.rfc.synchronize_step(0.5, 0.999, 0.001);
        })
    });

    c.bench_function("LOOP-STABILITY/full_lasing_step", |b| {
        b.iter(|| {
            controller.compute_lasing_force(&epsilon, &jacobian);
        })
    });
}

criterion_group!(benches, bench_loop_stability);
criterion_main!(benches);
