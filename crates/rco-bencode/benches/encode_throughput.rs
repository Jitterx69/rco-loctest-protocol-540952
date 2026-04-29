//! Criterion benchmark for RCO-Bencode encoding throughput.
//!
//! Measures:
//! - B-03: Bencode encode latency (target: < 500ns per batch)
//! - B-07: Key sort latency (target: < 50ns for 32 keys)

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rco_bencode::encoder::BencodeEncoder;
use rco_bencode::grammar::BencodeValue;

/// Builds a realistic telemetry batch with typical field count and sizes.
fn build_telemetry_batch() -> BencodeValue {
    let mut batch = BencodeValue::dict();
    batch
        .insert(b"action", BencodeValue::bytes(&[0x01, 0x02, 0x03, 0x04]))
        .unwrap();
    batch
        .insert(b"batch_index", BencodeValue::integer(42))
        .unwrap();
    batch
        .insert(b"observation", BencodeValue::bytes(&[0u8; 64]))
        .unwrap();
    batch
        .insert(b"reward", BencodeValue::integer(314_159_265_358_979_3_i128))
        .unwrap();
    batch
        .insert(
            b"run_uuid",
            BencodeValue::string("550e8400-e29b-41d4-a716-446655440000"),
        )
        .unwrap();
    batch
        .insert(b"timestamp", BencodeValue::bytes(&[0u8; 12]))
        .unwrap();
    batch
}

fn bench_encode_batch(c: &mut Criterion) {
    let batch = build_telemetry_batch();
    let mut buf = [0u8; 4096];

    c.bench_function("B-03: encode_telemetry_batch", |b| {
        b.iter(|| {
            let mut encoder = BencodeEncoder::new(&mut buf);
            encoder.encode(black_box(&batch)).unwrap();
        });
    });
}

fn bench_encode_integer(c: &mut Criterion) {
    let value = BencodeValue::integer(314_159_265_358_979_3_i128);
    let mut buf = [0u8; 64];

    c.bench_function("encode_p14_integer", |b| {
        b.iter(|| {
            let mut encoder = BencodeEncoder::new(&mut buf);
            encoder.encode(black_box(&value)).unwrap();
        });
    });
}

fn bench_encode_large_dict(c: &mut Criterion) {
    // 32-key dictionary to test sort performance
    let mut dict = BencodeValue::dict();
    for i in 0..32u8 {
        let key = [b'k', b'e', b'y', b'_', b'a' + i];
        dict.insert(&key, BencodeValue::integer(i128::from(i)))
            .unwrap();
    }
    let mut buf = [0u8; 8192];

    c.bench_function("B-07: encode_32key_dict", |b| {
        b.iter(|| {
            let mut encoder = BencodeEncoder::new(&mut buf);
            encoder.encode(black_box(&dict)).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_encode_batch,
    bench_encode_integer,
    bench_encode_large_dict,
);
criterion_main!(benches);
