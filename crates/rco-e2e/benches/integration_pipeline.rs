use criterion::{criterion_group, criterion_main, Criterion};
use rco_p14::projection::project_p14;
use rco_bencode::encoder::BencodeEncoder;
use rco_bencode::grammar::BencodeValue;
use rco_ingestion::pipeline::IngestionPipeline;
use rco_merkle::chain::compute_hash;
use tempfile::TempDir;
use std::time::Duration;

/// B-01: End-to-end ingestion latency (P14 -> Bencode -> Ingestion)
fn bench_e2e_pipeline(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("e2e.wal");
    let genesis_hash = compute_hash(b"genesis");
    
    let mut pipeline = IngestionPipeline::open_default(&wal_path, genesis_hash).unwrap();
    
    let mut group = c.benchmark_group("E2E Pipeline");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    let mut i = 1u64;
    group.bench_function("B-01_Full_Ingest", |b| {
        b.iter(|| {
            // 1. P14 Projection
            let reward = 3.14159265;
            let projected = project_p14(reward).unwrap();

            // 2. Bencode Serialization
            let mut buf = [0u8; 128];
            let node = BencodeValue::Integer(projected.raw());
            let mut encoder = BencodeEncoder::new(&mut buf);
            let written = encoder.encode(&node).unwrap();
            let encoded = &buf[..written];

            // 3. Ingestion (WAL + 2PC + RML)
            pipeline.ingest(i, encoded).unwrap();
            i += 1;
        })
    });

    group.finish();
}

criterion_group!(benches, bench_e2e_pipeline);
criterion_main!(benches);
