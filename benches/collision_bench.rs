// Run cargo bench to get benchmark results

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use map_builder_3d::collision::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut c_group = c.benchmark_group("collisions");

    let o1 = MoveableObject::new();
    let o2 = MoveableObject::new();

    // Change sample size
    c_group
        .sample_size(1000)
        .measurement_time(std::time::Duration::from_millis(500))
        .bench_function("moveable-moveable collides", |b| {
            b.iter(|| black_box(&o1).get_collition_with(black_box(&o2)))
        });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
