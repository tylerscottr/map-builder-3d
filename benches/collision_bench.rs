// Run cargo bench to get benchmark results

extern crate ncollide3d as nc3;

use map_builder_3d::collision::*;
use map_builder_3d::collision_walking::*;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut c_group = c.benchmark_group("collisions");

    let ball_left = WalkingObject::new(
        &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
        &nc3::na::Isometry3::<f32>::new(nc3::na::Vector3::<f32>::new(0., 0., 0.), nc3::na::zero()),
        &nc3::na::Vector3::<f32>::new(1., 0., 0.),
        &PositionOffset::Default,
    );
    let ball_right = WalkingObject::new(
        &Arc::new(ShapeType::Ball(nc3::shape::Ball::<f32>::new(1.))),
        &nc3::na::Isometry3::<f32>::new(nc3::na::Vector3::<f32>::new(10., 0., 0.), nc3::na::zero()),
        &nc3::na::Vector3::<f32>::new(-1., 0., 0.),
        &PositionOffset::Default,
    );

    // Change sample size
    c_group
        .sample_size(10000)
        .measurement_time(std::time::Duration::from_millis(500))
        .bench_function("moveable-moveable collides", |b| {
            b.iter(|| {
                black_box(&ball_left).get_collision_with(black_box(&ball_right), std::f32::MAX)
            })
        });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
