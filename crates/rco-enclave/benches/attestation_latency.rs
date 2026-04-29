use criterion::{criterion_group, criterion_main, Criterion};
use rco_enclave::ie::{RootOfTrustEnclave, AttestationQuote};

fn bench_attestation_latency(c: &mut Criterion) {
    let approved_mrenclave = [0xAA; 32];
    let rte = RootOfTrustEnclave::new(approved_mrenclave);
    let valid_quote = AttestationQuote {
        mrenclave: approved_mrenclave,
        mrsigner: [0xBB; 32],
        report_data: [0xCC; 32],
    };

    c.bench_function("ATTESTATION-LATENCY/verify_quote", |b| {
        b.iter(|| {
            rte.verify_ie_attestation(&valid_quote);
        })
    });
}

criterion_group!(benches, bench_attestation_latency);
criterion_main!(benches);
