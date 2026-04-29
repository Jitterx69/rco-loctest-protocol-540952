use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rco_audit::AuditScanner;
use rco_merkle::chain::{RmlChain, compute_hash};
use rco_ingestion::wal::WalEngine;
use tempfile::TempDir;
use std::time::Duration;

fn bench_audit_throughput(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bench.wal");
    let genesis = compute_hash(b"genesis");
    
    // Prepare 10,000 batches
    let batch_count = 10_000;
    {
        let mut wal = WalEngine::open(&path).unwrap();
        let mut chain = RmlChain::from_genesis(genesis);
        for i in 1..=batch_count {
            let data = b"i42e"; // Small batch
            let anchor = chain.extend(i, data).unwrap();
            let off = wal.prepare(i, data, &anchor.hash).unwrap();
            wal.commit(off).unwrap();
        }
    }

    let scanner = AuditScanner::new(genesis);

    let mut group = c.benchmark_group("Audit Sweep");
    group.throughput(criterion::Throughput::Elements(batch_count));
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(1));

    group.bench_function(BenchmarkId::new("TC-06", batch_count), |b| {
        b.iter(|| {
            let report = scanner.audit_wal(&path).unwrap();
            assert_eq!(report.verified_count, batch_count);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_audit_throughput);
criterion_main!(benches);
