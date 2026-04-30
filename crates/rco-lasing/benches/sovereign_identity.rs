use criterion::{criterion_group, criterion_main, Criterion};
use rco_enclave::attestation::AutonomousIdentity;

fn bench_sovereign_identity(c: &mut Criterion) {
    let root = [0u8; 32];
    let mut identity = AutonomousIdentity {
        manifold_id: [0u8; 32],
        generation: 1000,
    };
    
    // Set a valid manifold_id for benchmarking verification
    use sha3::{Keccak256, Digest};
    let mut hasher = Keccak256::new();
    hasher.update(b"RCO-AUTONOMOUS-IDENTITY-v4");
    hasher.update(&root);
    identity.manifold_id.copy_from_slice(&hasher.finalize());

    c.bench_function("SOVEREIGN-IDENTITY/aia_verification", |b| {
        b.iter(|| {
            let valid = identity.verify_sovereignty(&root);
            assert!(valid);
        })
    });
}

criterion_group!(benches, bench_sovereign_identity);
criterion_main!(benches);
