//! CONSENSUS-DRIFT Benchmark
//!
//! Simulates topological divergence under network jitter and Byzantine load.

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rco_consensus::drift::DriftDetector;

fn bench_consensus_drift(c: &mut Criterion) {
    let mut group = c.benchmark_group("CONSENSUS-DRIFT");
    
    group.bench_function("wasserstein_drift_evaluation", |b| {
        let detector = DriftDetector::new(0.005);
        let h1 = [0u8; 32];
        let h2 = [1u8; 32];
        
        b.iter(|| {
            // Level-5 check: Ensure drift detection is efficient
            let _report = black_box(detector.evaluate_node_drift(1, &h1, &h2, 0.0042));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_consensus_drift);
criterion_main!(benches);
