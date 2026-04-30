use criterion::{criterion_group, criterion_main, Criterion};
use rco_enclave::closure::SovereignClosure;

fn bench_sovereign_closure(c: &mut Criterion) {
    let mut closure = SovereignClosure::new();
    let state = vec![1.0; 1000];

    c.bench_function("SOVEREIGN-CLOSURE/self_attesting_root_generation", |b| {
        b.iter(|| {
            let root = closure.generate_self_attesting_root(&state);
            assert_eq!(root.len(), 32);
        })
    });
}

criterion_group!(benches, bench_sovereign_closure);
criterion_main!(benches);
