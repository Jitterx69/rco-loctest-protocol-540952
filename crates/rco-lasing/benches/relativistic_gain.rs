use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::controller::LasingController;
use nalgebra::DVector;

fn bench_relativistic_gain(c: &mut Criterion) {
    let dim = 1000;
    let mut controller = LasingController::new(dim, 10, 0.5, 1.0);
    let epsilon = DVector::from_element(10, 0.1);
    let jacobian = nalgebra::DMatrix::from_element(10, dim, 0.01);

    c.bench_function("RELATIVISTIC-GAIN/trans_oceanic_compensation", |b| {
        b.iter(|| {
            // Force magnitude simulates logical velocity/latency
            controller.compute_lasing_force(&epsilon, &jacobian);
        })
    });
}

criterion_group!(benches, bench_relativistic_gain);
criterion_main!(benches);
