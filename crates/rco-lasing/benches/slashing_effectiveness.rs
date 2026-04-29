use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::controller::LasingController;
use nalgebra::DVector;

fn bench_slashing_effectiveness(c: &mut Criterion) {
    let mut controller = LasingController::new(100, 10, 0.5, 1.0);
    let epsilon = DVector::from_element(10, 0.1);
    let jacobian = nalgebra::DMatrix::from_element(10, 100, 0.01);

    c.bench_function("SLASHING-EFFECTIVENESS/neutralize_shard", |b| {
        b.iter(|| {
            // Simulate immediate slashing (muting)
            controller.global_slashing = 0.0;
            let force = controller.compute_lasing_force(&epsilon, &jacobian);
            assert_eq!(force.norm(), 0.0);
            
            // Restore for next iteration
            controller.global_slashing = 1.0;
        })
    });
}

criterion_group!(benches, bench_slashing_effectiveness);
criterion_main!(benches);
