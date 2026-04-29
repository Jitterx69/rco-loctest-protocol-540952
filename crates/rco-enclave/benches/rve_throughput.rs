//! RVE-THROUGHPUT Benchmark
//!
//! Simulates C-502 tests for Ingestion Enclave (IE) throughput and pruning.

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rco_enclave::ie::{IngestionEnclave, IEPoint};

fn bench_rve_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("RVE-THROUGHPUT");
    
    group.bench_function("ie_ingestion_sps_c502", |b| {
        // We simulate ingesting a batch of points to measure SPS
        b.iter(|| {
            let mut ie = IngestionEnclave::new();
            for i in 0..10_000 {
                ie.ingest_step(black_box(IEPoint {
                    step: i as u64,
                    state_hash: [0x11; 32],
                    is_landmark: i % 100 == 0,
                }));
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_rve_throughput);
criterion_main!(benches);
