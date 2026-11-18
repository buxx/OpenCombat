use gungraun::prelude::*;
use oc_root::geo::WorldVec2;
use oc_utils::polygon::Polygon;
use std::hint::black_box;

struct Poly(Vec<WorldVec2>);

impl Polygon for Poly {
    fn points(&self) -> &[WorldVec2] {
        &self.0
    }
}

fn square() -> Poly {
    Poly(vec![
        WorldVec2::new(0., 0.),
        WorldVec2::new(100., 0.),
        WorldVec2::new(100., 100.),
        WorldVec2::new(0., 100.),
    ])
}

/// Star-ish concave polygon with many vertices
fn complex() -> Poly {
    Poly(
        (0..64)
            .map(|i| {
                let angle = i as f32 / 64. * std::f32::consts::TAU;
                let radius = if i % 2 == 0 { 100. } else { 60. };
                WorldVec2::new(150. + radius * angle.cos(), 150. + radius * angle.sin())
            })
            .collect(),
    )
}

fn with_square(point: WorldVec2) -> (Poly, WorldVec2) {
    (square(), point)
}

fn with_complex(point: WorldVec2) -> (Poly, WorldVec2) {
    (complex(), point)
}

#[library_benchmark]
#[bench::inside(setup = with_square, args = (WorldVec2::new(50., 50.),))]
#[bench::outside(setup = with_square, args = (WorldVec2::new(150., 50.),))]
fn bench_contains_square((poly, point): (Poly, WorldVec2)) -> bool {
    black_box(poly.contains(black_box(point)))
}

#[library_benchmark]
#[bench::inside(setup = with_complex, args = (WorldVec2::new(150., 150.),))]
#[bench::outside(setup = with_complex, args = (WorldVec2::new(0., 0.),))]
fn bench_contains_complex((poly, point): (Poly, WorldVec2)) -> bool {
    black_box(poly.contains(black_box(point)))
}

library_benchmark_group!(
    name = bench_polygon_contains_group,
    benchmarks = [bench_contains_square, bench_contains_complex]
);

main!(library_benchmark_groups = bench_polygon_contains_group);
