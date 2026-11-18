use glam::Vec2;
use gungraun::prelude::*;
use oc_utils::d2::{Angle, Rect, Shape};
use std::hint::black_box;

fn shape() -> Shape {
    Shape::from_rect(&Rect {
        x: 10.,
        y: 10.,
        w: 40.,
        h: 20.,
    })
}

#[library_benchmark]
#[bench::quarter_turn(Angle(1.57))]
#[bench::arbitrary(Angle(0.7))]
fn bench_shape_rotate(angle: Angle) -> Shape {
    black_box(shape().rotate(black_box(&angle)))
}

#[library_benchmark]
#[bench::inside(Vec2::new(30., 20.))]
#[bench::outside(Vec2::new(300., 200.))]
fn bench_shape_contains(point: Vec2) -> bool {
    black_box(shape().contains(black_box(&point)))
}

library_benchmark_group!(
    name = bench_shape_group,
    benchmarks = [bench_shape_rotate, bench_shape_contains]
);

main!(library_benchmark_groups = bench_shape_group);
