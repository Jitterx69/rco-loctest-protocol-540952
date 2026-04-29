use criterion::{criterion_group, criterion_main, Criterion};
use rco_enclave::synthesis::HardwareBoundSynthesis;

fn bench_hardware_synthesis(c: &mut Criterion) {
    let synthesis = HardwareBoundSynthesis::new();
    let state_hash = [0u8; 32];

    c.bench_function("HARDWARE-SYNTHESIS/physical_attestation", |b| {
        b.iter(|| {
            let attestation = synthesis.attest_physical_invariance(&state_hash);
            assert_ne!(attestation, [0u8; 32]);
        })
    });
}

criterion_group!(benches, bench_hardware_synthesis);
criterion_main!(benches);
