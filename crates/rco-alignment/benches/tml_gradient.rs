//! TML-GRADIENT Benchmark
//!
//! Evaluates the efficiency of the Acceleration-Based Surrogate (ABS) and Damped Lasering.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use nalgebra::{DMatrix, DVector};
use rco_alignment::lasering::{LaseringConfig, apply_omega_lasering};
use rco_alignment::simplicial::SimplicialOmegaKernel;

fn bench_tml_gradient(c: &mut Criterion) {
    let mut group = c.benchmark_group("TML-OMEGA-GRADIENT");

    group.bench_function("simplicial_omega_smoothing", |b| {
        // Initialize a 6x6 boundary matrix for a small simplicial complex
        let b1 = DMatrix::from_element(6, 6, 0.0);
        let kernel = SimplicialOmegaKernel::new(b1);
        let flow = DVector::from_element(6, 0.5);
        let lambda = 0.08;

        b.iter(|| {
            // Benchmark the high-frequency Ricci-weighted smoothing operation
            let _smoothed = black_box(kernel.hyper_smooth(&flow, lambda));
        })
    });

    group.bench_function("omega_lasering_update", |b| {
        let config = LaseringConfig::default();
        let mut state = DVector::from_vec(vec![0.0; 256]);
        let mut velocity = DVector::from_vec(vec![0.1; 256]);
        let topo_grad = DVector::from_vec(vec![0.05; 256]);
        let align_vec = DVector::from_vec(vec![1.0; 256]);
        let ricci_laplacian = DVector::from_vec(vec![0.02; 256]);

        b.iter(|| {
            apply_omega_lasering(
                black_box(&mut state),
                black_box(&topo_grad),
                black_box(&align_vec),
                black_box(&ricci_laplacian),
                black_box(&mut velocity),
                &config,
            );
        })
    });

    group.finish();
}

criterion_group!(benches, bench_tml_gradient);
criterion_main!(benches);
