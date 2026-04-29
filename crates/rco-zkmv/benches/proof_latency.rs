//! ZK-MV Proof Latency Benchmark
//!
//! Measures SNARK generation and verification overhead for manifold stability.

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rco_zkmv::proof::{generate_manifold_proof, verify_manifold_proof};
use ark_groth16::Groth16;
use ark_bls12_381::Bls12_381;
use ark_snark::SNARK;
use rand::thread_rng;
use rco_zkmv::constraints::CoherenceCircuit;

fn bench_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ZK-MV");
    
    group.bench_function("generate_proof", |b| {
        b.iter(|| {
            let _proof = black_box(generate_manifold_proof(1000, 999));
        })
    });

    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(
        CoherenceCircuit { coherence: None, threshold: None }, 
        &mut rng
    ).unwrap();
    let proof = generate_manifold_proof(1000, 999);

    group.bench_function("verify_proof", |b| {
        b.iter(|| {
            let _valid = black_box(verify_manifold_proof(&proof, &vk, &[1000, 999]));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_proof_generation);
criterion_main!(benches);
