//! ZK-MV Proof Latency Benchmark
//!
//! Measures SNARK generation and verification overhead for manifold stability.

use ark_bls12_381::Bls12_381;
use ark_groth16::Groth16;
use ark_snark::SNARK;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rand::thread_rng;
use rco_zkmv::constraints::CoherenceCircuit;
use rco_zkmv::proof::{generate_forensic_proof, verify_forensic_proof};

fn bench_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ZK-MV-LATENCY");

    let projection = 1000u64;
    let entropy = 999u64;
    let threshold = 995u64;
    let hardware_id = 0xDEADBEEFu64;

    // Use the forensic generator to get the proving/verifying keys
    // In a benchmark, we want to isolate the proof generation/verification
    let (_initial_proof, pk, vk) =
        generate_forensic_proof(projection, entropy, threshold, hardware_id);

    group.bench_function("generate_forensic_proof", |b| {
        b.iter(|| {
            // Measure pure proof generation overhead
            let _proof = black_box(generate_forensic_proof(
                projection,
                entropy,
                threshold,
                hardware_id,
            ));
        })
    });

    let (proof, _, _) = generate_forensic_proof(projection, entropy, threshold, hardware_id);

    group.bench_function("verify_forensic_proof", |b| {
        b.iter(|| {
            // Measure pure verification overhead against public inputs
            let _valid = black_box(verify_forensic_proof(&proof, &vk, threshold, hardware_id));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_proof_generation);
criterion_main!(benches);
