//! RVE-ISOLATION Benchmark
//!
//! Simulates C-501 and C-503 tests for Root-of-Trust Enclave isolation.

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rco_enclave::shunt::{SecureShunt};

fn bench_rve_isolation(c: &mut Criterion) {
    let mut group = c.benchmark_group("RVE-ISOLATION");
    
    group.bench_function("memory_scan_leakage_c501", |b| {
        let shunt = SecureShunt::new();
        let secret_root = [0xAA; 32];
        shunt.rte_write_root(&secret_root);
        
        b.iter(|| {
            // Host attempts to read the RTE register directly
            let result = black_box(shunt.host_read_rte_root());
            // It MUST return an error for perfect isolation
            assert!(result.is_err());
        })
    });

    group.finish();
}

criterion_group!(benches, bench_rve_isolation);
criterion_main!(benches);
