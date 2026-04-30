use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::controller::LasingController;
use nalgebra::DVector;

fn bench_meta_stability(c: &mut Criterion) {
    let dim = 1000;
    let mut controller = LasingController::new(dim, 10, 0.5, 1.0);
    let epsilon = DVector::from_element(10, 0.1);
    let jacobian = nalgebra::DMatrix::from_element(10, dim, 0.01);

    c.bench_function("META-STABILITY/evolutionary_convergence", |b| {
        b.iter(|| {
            controller.compute_lasing_force(&epsilon, &jacobian);
        })
    });
}

criterion_group!(benches, bench_meta_stability);
criterion_main!(benches);
