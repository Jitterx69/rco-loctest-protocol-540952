use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::controller::LasingController;
use nalgebra::{DMatrix, DVector};

fn bench_omega_invariance(c: &mut Criterion) {
    let dim = 1000;
    let mut controller = LasingController::new(dim, 10, 0.5, 1.0);
    let epsilon = DVector::from_element(10, 0.1);
    let jacobian = nalgebra::DMatrix::from_element(10, dim, 0.01);

    // Force Omega Point
    controller.omega_achieved = true;
    controller.omega_finality.is_locked = true;

    c.bench_function("OMEGA-INVARIANCE/gradient_neutralization", |b| {
        b.iter(|| {
            let force = controller.compute_lasing_force(&epsilon, &jacobian);
            assert_eq!(force.norm(), 0.0);
        })
    });
}

criterion_group!(benches, bench_omega_invariance);
criterion_main!(benches);
