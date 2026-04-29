use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::controller::LasingController;
use nalgebra::DVector;

fn bench_planetary_sync(c: &mut Criterion) {
    let dim = 1000;
    let mut controller = LasingController::new(dim, 10, 0.5, 1.0);
    let epsilon = DVector::from_element(10, 0.0); // Stale/delayed data (zero norm)
    let jacobian = nalgebra::DMatrix::from_element(10, dim, 0.01);

    // Populate LEE buffer
    for i in 0..10 {
        controller.lee.buffer.push(DVector::from_element(dim, i as f64 * 0.1));
    }
    controller.lee.holographic_reconstruction(0.001);

    c.bench_function("PLANETARY-SYNC/latent_emulation_loop", |b| {
        b.iter(|| {
            // Compute force using synthetic gradients from LEE due to simulated delay
            let force = controller.compute_lasing_force(&epsilon, &jacobian);
            assert!(force.norm() > 0.0);
        })
    });
}

criterion_group!(benches, bench_planetary_sync);
criterion_main!(benches);
