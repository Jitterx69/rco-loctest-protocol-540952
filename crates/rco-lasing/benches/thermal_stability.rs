use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::controller::LasingController;
use nalgebra::DVector;

fn bench_thermal_stability(c: &mut Criterion) {
    let dim = 1000;
    let mut controller = LasingController::new(dim, 10, 0.5, 1.0);
    let epsilon = DVector::from_element(10, 0.1);
    let jacobian = nalgebra::DMatrix::from_element(10, dim, 0.01);

    c.bench_function("THERMAL-STABILITY/gain_scheduling_1.0K_to_4.0K", |b| {
        b.iter(|| {
            // Simulate evolution and thermal shift
            controller.evolution.generation += 1000;
            controller.compute_lasing_force(&epsilon, &jacobian);
        })
    });
}

criterion_group!(benches, bench_thermal_stability);
criterion_main!(benches);
