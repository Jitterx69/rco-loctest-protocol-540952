//! TML-GRADIENT Benchmark
//!
//! Evaluates the efficiency of the Acceleration-Based Surrogate (ABS) and Damped Lasering.

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rco_alignment::simplicial::AccelerationSurrogate;
use rco_alignment::lasering::{apply_damped_lasering, LaseringConfig};
use nalgebra::DVector;

fn bench_tml_gradient(c: &mut Criterion) {
    let mut group = c.benchmark_group("TML-GRADIENT");
    
    group.bench_function("simplicial_abs_surrogate", |b| {
        let abs = AccelerationSurrogate::new(1e-6);
        let velocity = DVector::from_vec(vec![1.0, 0.5, -0.2, 0.4, 0.1, -0.8]);
        let accel = DVector::from_vec(vec![0.1, -0.1, 0.05, 0.01, 0.0, 0.05]);
        let entropy = 1.2;
        
        b.iter(|| {
            let _loss = black_box(abs.compute_loss(&velocity, &accel, entropy));
            let _grad = black_box(abs.compute_gradient(&velocity, &accel, entropy));
        })
    });
    
    group.bench_function("damped_lasering_update", |b| {
        let config = LaseringConfig::default();
        let mut state = DVector::from_vec(vec![0.0; 256]);
        let mut velocity = DVector::from_vec(vec![0.1; 256]);
        let topo_grad = DVector::from_vec(vec![0.05; 256]);
        let align_vec = DVector::from_vec(vec![1.0; 256]);
        
        b.iter(|| {
            apply_damped_lasering(
                black_box(&mut state),
                black_box(&topo_grad),
                black_box(&align_vec),
                black_box(&mut velocity),
                &config
            );
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_tml_gradient);
criterion_main!(benches);
