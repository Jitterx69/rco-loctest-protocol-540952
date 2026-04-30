use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::correction::RecursiveSelfCorrection;
use nalgebra::DVector;

fn bench_self_correction(c: &mut Criterion) {
    let mut correction = RecursiveSelfCorrection::new();
    let mut state = DVector::from_element(1000, 1.0);

    c.bench_function("SELF-CORRECTION/topological_feedback_snapback", |b| {
        b.iter(|| {
            correction.apply_topological_feedback(&mut state, 0.5);
        })
    });
}

criterion_group!(benches, bench_self_correction);
criterion_main!(benches);
