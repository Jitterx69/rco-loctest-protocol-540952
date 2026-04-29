//! JACOBIAN-CONVERGENCE Benchmark (B-II-02)
//!
//! Evaluates J_re update latency for high-dimensional parameter spaces.

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use nalgebra::{DMatrix, DVector};
use rco_reflexive::jacobian::ReflexiveJacobian;

fn bench_jacobian_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("JACOBIAN-CONVERGENCE");
    
    // Scale from 1k to 10k parameters for local benchmarking
    for dim in [1000, 2000, 5000].iter() {
        group.bench_with_input(format!("update_{}_params", dim), dim, |b, &dim| {
            let obs_dim = 100;
            let rj = ReflexiveJacobian::new(dim, obs_dim);
            let epsilon = DVector::from_element(obs_dim, 0.01);
            let jacobian = DMatrix::from_element(obs_dim, dim, 0.1);

            b.iter(|| {
                let _update = black_box(rj.compute_update(&epsilon, &jacobian));
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_jacobian_update);
criterion_main!(benches);
