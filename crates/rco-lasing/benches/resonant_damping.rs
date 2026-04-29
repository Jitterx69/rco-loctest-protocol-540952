use criterion::{criterion_group, criterion_main, Criterion};
use rco_lasing::damper::ActiveResonantDamper;
use nalgebra::DVector;

fn bench_resonant_damping(c: &mut Criterion) {
    let mut damper = ActiveResonantDamper::new();
    let telemetry = DVector::from_element(10, 1.0); // Normal
    let stress_telemetry = DVector::from_element(10, 10.0); // Resonance trigger

    c.bench_function("RESONANT-DAMPING/echo_detection", |b| {
        b.iter(|| {
            damper.detect_echoes(&telemetry);
        })
    });

    c.bench_function("RESONANT-DAMPING/counter_pulse_generation", |b| {
        let echoes = damper.detect_echoes(&stress_telemetry);
        b.iter(|| {
            damper.generate_counter_pulse(&echoes);
        })
    });
}

criterion_group!(benches, bench_resonant_damping);
criterion_main!(benches);
