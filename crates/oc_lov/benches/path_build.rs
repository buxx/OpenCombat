use gungraun::prelude::*;
use oc_lov::{Modifier, Path, PathBuilder, Step};
use oc_root::{WorldConfig, opacity::Opacity};
use std::hint::black_box;

fn world() -> WorldConfig {
    WorldConfig::new(200, 1).with_geo_pixels_per_tile(5)
}

fn step(opacity: f32) -> Step {
    Step {
        opacity: Opacity(opacity),
        crawling_opacity_bonus: 0.0,
        hiding_opacity_bonus: 0.0,
    }
}

#[library_benchmark]
#[bench::short([0., 0., 0.], [14., 0., 0.])]
#[bench::long_flat([0., 0., 0.], [900., 300., 10.])]
#[bench::long_diagonal([0., 0., 0.], [900., 900., 0.])]
fn bench_build_transparent(start: [f32; 3], end: [f32; 3]) -> Path {
    let w = world();
    let at = |_, _| Some(step(0.0));
    black_box(PathBuilder::new(&w, at).build(
        black_box(start.into()),
        black_box(end.into()),
        0,
        Modifier::None(false),
    ))
}

#[library_benchmark]
#[bench::light_obstacles([0., 0., 0.], [900., 300., 10.])]
#[bench::long_diagonal([0., 0., 0.], [900., 900., 0.])]
fn bench_build_partly_opaque(start: [f32; 3], end: [f32; 3]) -> Path {
    let w = world();
    let at = |_, _| Some(step(0.01));
    black_box(PathBuilder::new(&w, at).build(
        black_box(start.into()),
        black_box(end.into()),
        0,
        Modifier::None(true),
    ))
}

library_benchmark_group!(
    name = bench_path_build_group,
    benchmarks = [bench_build_transparent, bench_build_partly_opaque]
);

main!(library_benchmark_groups = bench_path_build_group);
