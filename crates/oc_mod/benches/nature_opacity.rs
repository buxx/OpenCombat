use gungraun::prelude::*;
use oc_mod::{Mod, nature::NatureIndex};
use oc_root::WorldConfig;
use std::{hint::black_box, path::PathBuf};

fn world() -> WorldConfig {
    WorldConfig::new(100, 100).with_geo_pixels_per_tile(5)
}

fn mod_() -> Mod {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods/tests1");
    Mod::load(&path, None).unwrap()
}

fn with_mod(relative_z: f32) -> (Mod, WorldConfig, f32) {
    (mod_(), world(), relative_z)
}

// Called once by tile crossed by each line of view
#[library_benchmark]
#[bench::in_ground(setup = with_mod, args = (-1.,))]
#[bench::in_object(setup = with_mod, args = (1.,))]
#[bench::in_air(setup = with_mod, args = (1000.,))]
fn bench_nature_opacity((mod_, w, relative_z): (Mod, WorldConfig, f32)) -> (Mod, f32) {
    // Mod returned to be dropped outside of the measure
    let opacity = mod_
        .nature(NatureIndex(0))
        .opacity(black_box(&w), black_box(relative_z))
        .0;
    (mod_, black_box(opacity))
}

library_benchmark_group!(
    name = bench_nature_opacity_group,
    benchmarks = [bench_nature_opacity]
);

main!(library_benchmark_groups = bench_nature_opacity_group);
