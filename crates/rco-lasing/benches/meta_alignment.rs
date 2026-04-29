use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::controller::LasingController;
use nalgebra::{DVector, DMatrix};

fn bench_meta_alignment(c: &mut Criterion) {
    let dim = 1000;
    let mut controller = LasingController::new(dim, 10, 0.5, 1.0);
    let epsilon = DVector::from_element(10, 0.1);
    let jacobian = DMatrix::from_element(10, dim, 0.01);

    c.bench_function("META-ALIGNMENT/ricci_flux_anticipation", |b| {
        b.iter(|| {
            // Force the Meta-Reflexive loop to adjust gain based on simulated Ricci Flux
            controller.compute_lasing_force(&epsilon, &jacobian);
        })
    });
}

criterion_group!(benches, bench_meta_alignment);
criterion_main!(benches);
