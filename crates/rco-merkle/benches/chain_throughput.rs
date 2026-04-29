//! Criterion benchmark for RML chain throughput.
//!
//! Measures:
//! - B-05: RML chain extend latency (target: < 200ns per step)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rco_merkle::chain::{compute_hash, RmlChain};

fn bench_chain_extend(c: &mut Criterion) {
    let genesis = compute_hash(b"benchmark_genesis");
    let batch_data = [0x42u8; 256]; // Typical encoded batch size

    c.bench_function("B-05: rml_chain_extend", |b| {
        let mut chain = RmlChain::from_genesis(genesis);
        let mut idx = 1u64;
        b.iter(|| {
            chain.extend(idx, black_box(&batch_data)).unwrap();
            idx += 1;
        });
    });
}

fn bench_compute_hash(c: &mut Criterion) {
    let data = [0x42u8; 256];

    c.bench_function("keccak256_standalone", |b| {
        b.iter(|| compute_hash(black_box(&data)));
    });
}

criterion_group!(benches, bench_chain_extend, bench_compute_hash);
criterion_main!(benches);
