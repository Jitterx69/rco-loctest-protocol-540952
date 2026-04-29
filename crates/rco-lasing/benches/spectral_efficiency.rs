//! SPECTRAL-EFFICIENCY Benchmark
//!
//! Evaluates cross-shard isolation (Sigma) across high-density manifolds.

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use nalgebra::DVector;
use rco_lasing::spectral::SpectralMonitor;

fn bench_spectral_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("SPECTRAL-EFFICIENCY");
    
    group.bench_function("bleed_calculation", |b| {
        let s1 = DVector::from_element(1000, 1.0);
        let s2 = DVector::from_element(1000, 0.01);
        
        b.iter(|| {
            black_box(SpectralMonitor::calculate_bleed(&s1, &s2));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_spectral_efficiency);
criterion_main!(benches);
