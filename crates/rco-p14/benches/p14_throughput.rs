//! Criterion benchmark for P14 projection throughput.
//!
//! Measures:
//! - B-04: P14 projection latency (target: < 5ns per scalar)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rco_p14::boundary::{classify_reward, flush_subnormals};
use rco_p14::projection::project_p14;

fn bench_project_p14_single(c: &mut Criterion) {
    let r = core::f64::consts::PI;

    c.bench_function("B-04: project_p14_single", |b| {
        b.iter(|| project_p14(black_box(r)).unwrap());
    });
}

fn bench_project_p14_batch(c: &mut Criterion) {
    let rewards: Vec<f64> = (0..1000).map(|i| (i as f64) * 0.001 - 0.5).collect();

    c.bench_function("project_p14_batch_1000", |b| {
        b.iter(|| {
            for &r in black_box(&rewards) {
                let _ = project_p14(r).unwrap();
            }
        });
    });
}

fn bench_classify_reward(c: &mut Criterion) {
    let r = 42.5_f64;

    c.bench_function("classify_reward", |b| {
        b.iter(|| classify_reward(black_box(r)));
    });
}

fn bench_flush_subnormals(c: &mut Criterion) {
    let r = 5e-324_f64;

    c.bench_function("flush_subnormals", |b| {
        b.iter(|| flush_subnormals(black_box(r)));
    });
}

criterion_group!(
    benches,
    bench_project_p14_single,
    bench_project_p14_batch,
    bench_classify_reward,
    bench_flush_subnormals,
);
criterion_main!(benches);
