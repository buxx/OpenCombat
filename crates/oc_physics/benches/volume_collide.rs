use glam::Quat;
use gungraun::prelude::*;
use oc_physics::volume::Volume;
use std::hint::black_box;

fn cube(x: f32, y: f32, z: f32) -> Volume {
    Volume::Cube {
        x,
        y,
        z,
        width: 10.,
        height: 10.,
        depth: 10.,
    }
}

fn rot(angle: f32) -> Quat {
    Quat::from_rotation_z(angle)
}

#[library_benchmark]
#[bench::overlap_rotated(cube(0., 0., 0.), rot(0.5), cube(5., 5., 5.), rot(1.2))]
#[bench::disjoint_close(cube(0., 0., 0.), rot(0.5), cube(12., 0., 0.), rot(1.2))]
#[bench::disjoint_far(cube(0., 0., 0.), Quat::IDENTITY, cube(500., 500., 0.), Quat::IDENTITY)]
fn bench_cube_cube(a: Volume, ra: Quat, b: Volume, rb: Quat) -> bool {
    black_box(a.collide(black_box(ra), black_box(&b), black_box(rb)))
}

#[library_benchmark]
#[bench::inside(Volume::Point { x: 5., y: 5., z: 5. }, cube(0., 0., 0.), rot(0.7))]
#[bench::outside(Volume::Point { x: 50., y: 5., z: 5. }, cube(0., 0., 0.), rot(0.7))]
fn bench_point_cube(point: Volume, cube: Volume, rotation: Quat) -> bool {
    black_box(point.collide(Quat::IDENTITY, black_box(&cube), black_box(rotation)))
}

library_benchmark_group!(
    name = bench_volume_collide_group,
    benchmarks = [bench_cube_cube, bench_point_cube]
);

main!(library_benchmark_groups = bench_volume_collide_group);
