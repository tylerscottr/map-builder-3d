// Run cargo bench to get benchmark results

extern crate ncollide3d as nc3;

use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use map_builder_3d::collision::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut c_group = c.benchmark_group("collisions");

    let o1 = WalkingObject::new(Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))));
    let o2 = WalkingObject::new(Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))));

    // Change sample size
    c_group
        .sample_size(1000)
        .measurement_time(std::time::Duration::from_millis(500))
        .bench_function("moveable-moveable collides", |b| {
            b.iter(|| black_box(&o1).get_collision_with(black_box(&o2)))
        });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
