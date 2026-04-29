use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::controller::LasingController;
use nalgebra::{DVector, DMatrix};

fn bench_final_audit(c: &mut Criterion) {
    let dim = 1000;
    let mut controller = LasingController::new(dim, 10, 0.5, 1.0);
    let epsilon = DVector::from_element(10, 0.05);
    let jacobian = DMatrix::from_element(10, dim, 0.01);

    c.bench_function("FINAL-AUDIT/global_convergence_step", |b| {
        b.iter(|| {
            controller.compute_lasing_force(&epsilon, &jacobian);
        })
    });
}

criterion_group!(benches, bench_final_audit);
criterion_main!(benches);
