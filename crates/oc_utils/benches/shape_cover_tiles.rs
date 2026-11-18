use gungraun::prelude::*;
use oc_utils::d2::shape_cover_tiles;
use std::hint::black_box;

#[library_benchmark]
fn bench_shape_cover_tiles_with_little_shape() {
    shape_cover_tiles(
        black_box([0., 0.]),
        black_box(2.),
        black_box(2.),
        black_box(5.),
        black_box(5.),
    );
}

#[library_benchmark]
fn bench_shape_cover_tiles_with_big_shape() {
    shape_cover_tiles(
        black_box([0., 0.]),
        black_box(100.),
        black_box(100.),
        black_box(5.),
        black_box(5.),
    );
}

library_benchmark_group!(
    name = bench_shape_cover_tiles_group,
    benchmarks = [
        bench_shape_cover_tiles_with_little_shape,
        bench_shape_cover_tiles_with_big_shape
    ]
);

main!(library_benchmark_groups = bench_shape_cover_tiles_group);
