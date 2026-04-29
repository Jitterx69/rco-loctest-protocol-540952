//! CONSENSUS-LATENCY Benchmark
//!
//! Measures time from simplicial contribution to QBM finality.

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rco_consensus::qbm::{QBMConstructor, Simplex};
use rco_consensus::sba::SBAProtocol;

fn bench_consensus_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("CONSENSUS-LATENCY");
    
    group.bench_function("qbm_finality_64_nodes_100k_simplices", |b| {
        let n = 64;
        let mut constructor = QBMConstructor::new(n);
        let simplices: Vec<Simplex> = (0..1000).map(|i| Simplex { dim: 0, vertices: vec![i] }).collect();
        
        b.iter(|| {
            // Simulate contribution from all 64 nodes
            for _ in 0..n {
                constructor.register_node_contribution(black_box(simplices.clone()));
            }
            
            // Finalize QBM
            let qbm = black_box(constructor.finalize_qbm());
            
            // SBA voting on the QBM-Root
            let mut sba = SBAProtocol::new(n);
            let root = constructor.compute_qbm_root(&qbm);
            for i in 0..n {
                sba.cast_vote(i as u64, root);
            }
            
            let _consensus = black_box(sba.check_consensus());
        })
    });

    group.finish();
}

criterion_group!(benches, bench_consensus_latency);
criterion_main!(benches);
