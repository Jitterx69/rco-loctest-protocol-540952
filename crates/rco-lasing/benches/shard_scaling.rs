use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rco_lasing::controller::LasingController;
use nalgebra::{DVector, DMatrix};

fn bench_shard_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("SHARD-SCALING");
    
    for size in [1000, 5000, 10000].iter() {
        let dim = *size;
        let mut controller = LasingController::new(dim, 10, 0.5, 1.0);
        let epsilon = DVector::from_element(10, 0.1);
        let jacobian = DMatrix::from_element(10, dim, 0.01);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                controller.compute_lasing_force(&epsilon, &jacobian);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_shard_scaling);
criterion_main!(benches);
