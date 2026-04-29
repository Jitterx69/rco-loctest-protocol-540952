//! THRESHOLD-FINALITY Benchmark
//!
//! Measures BLS aggregation latency for varying node counts.

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use bls12_381::{G1Projective, G2Projective, Scalar};
use group::Group;
use ff::Field;
use rand::thread_rng;
use rco_threshold::tmpq::{TMPQ, PartialSignature};
use rco_threshold::sss::generate_shares;

fn bench_threshold_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("THRESHOLD-FINALITY");
    
    for n in [64, 128, 256].iter() {
        group.bench_with_input(format!("aggregate_{}_nodes", n), n, |b, &n| {
            let mut rng = thread_rng();
            let secret = Scalar::random(&mut rng);
            let message_hash = G2Projective::random(&mut rng);
            let t = (2 * n) / 3;
            let tmpq = TMPQ::new(t, n);
            let shares = generate_shares(secret, t, n, &mut rng);
            let partials: Vec<PartialSignature> = shares[0..t]
                .iter()
                .map(|s| tmpq.sign_partial(s, message_hash))
                .collect();

            b.iter(|| {
                let _sig = black_box(tmpq.aggregate(&partials));
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_threshold_aggregation);
criterion_main!(benches);
