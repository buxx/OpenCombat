use gungraun::prelude::*;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_mod::{
    Mod,
    nature::{NatureIndex, Traversability},
};
use oc_physics::{Event, Force, IgnoreSide, Physic, collision::Material, volume::Volume};
use oc_root::{
    WcfgFrom, WorldConfig, geo::WorldVec3, material::MaterialKind, physics::MetersSeconds,
    side::Side,
};
use oc_utils::d2::{Direction, Xy};
use oc_world::{World, tile::Tile};
use oc_world_server::{index::Indexes, visibility::Processor};
use serde::Serialize;
use std::{hint::black_box, path::PathBuf};
use tests::{individual::TestIndividual, world::TestWorld};

const SIDE_COUNT: usize = 10;

fn mod_() -> Mod {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods/tests1");
    Mod::load(&path, None).unwrap()
}

fn world_config() -> WorldConfig {
    WorldConfig::new(100, 100).with_geo_pixels_per_tile(5)
}

/// `SIDE_COUNT` individuals by side, side A on the left, side B on the right.
fn battle_world() -> (World, WorldConfig) {
    let w = world_config();
    let individuals = (0..SIDE_COUNT * 2)
        .map(|i| {
            let (side, x) = if i % 2 == 0 {
                (Side::A, 20.)
            } else {
                (Side::B, 400.)
            };
            TestIndividual::builder()
                .side(side)
                .position(WorldVec3::new(x, 20. + (i / 2) as f32 * 30., 0.))
                .build()
                .make(&w)
        })
        .collect();
    let world = TestWorld::builder()
        .individuals(individuals)
        .build()
        .make(&w);
    (world, w)
}

// World indexation, rebuilt at each tick
#[library_benchmark]
#[bench::battle(setup = battle_world)]
fn bench_indexes_new((world, w): (World, WorldConfig)) -> (World, Indexes) {
    // World returned to be dropped outside of the measure
    let indexes = Indexes::new(black_box(&world), black_box(&w));
    (world, black_box(indexes))
}

// Visibilities of all opposite individuals pairs
#[library_benchmark]
#[bench::battle(setup = battle_world)]
fn bench_visibility_compute((world, w): (World, WorldConfig)) -> (World, Indexes) {
    let indexes = Indexes::new(&world, &w);
    black_box(Processor::new(&world, &indexes).compute());
    (world, indexes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum ObjectsId {
    Tile(WorldTileIndex),
    Object,
}

struct Bullet(WorldVec3, Vec<Force>);

impl Material for Bullet {
    fn kind(&self) -> Option<MaterialKind> {
        Some(MaterialKind::Projectile)
    }
}

impl Physic for Bullet {
    fn position(&self, _: &WorldConfig) -> WorldVec3 {
        self.0
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        &self.1
    }

    fn volumes(
        &self,
        ref_: WorldVec3,
        _: bool,
        _: &WorldConfig,
        _: &Mod,
    ) -> Vec<(Volume, Traversability, Direction)> {
        vec![(
            Volume::Point {
                x: ref_.x,
                y: ref_.y,
                z: ref_.z,
            },
            Traversability::all(),
            Direction::NORTH,
        )]
    }

    fn ignore_side(&self) -> IgnoreSide {
        IgnoreSide::None
    }
}

struct StepInput {
    w: WorldConfig,
    mod_: Mod,
    tiles: Vec<Tile>,
    bullet: Bullet,
}

/// Bullet flying over `tiles_z` high tiles floor
fn step_input(direction: [f32; 3], tile_z: u8) -> StepInput {
    let w = WorldConfig::new(100, 100)
        .with_geo_pixels_per_tile(5)
        .with_geo_pixels_per_meters(5.);
    let mod_ = mod_();
    let traversability = mod_.nature(NatureIndex(0)).traversability.clone();
    let tiles = (0..w.tiles_count())
        .map(|i| {
            Tile::new(
                WorldTileIndex(i),
                NatureIndex(0),
                tile_z,
                traversability.clone(),
            )
        })
        .collect();
    let bullet = Bullet(
        WorldVec3::new(2.5, 2.5, 20.),
        vec![Force::Translation(direction.into(), MetersSeconds(200.))],
    );
    StepInput {
        w,
        mod_,
        tiles,
        bullet,
    }
}

// One physic step of a projectile, the most frequent physics computation
#[library_benchmark]
#[bench::flying(setup = step_input, args = ([1., 0.2, 0.], 0))]
#[bench::hitting_ground(setup = step_input, args = ([1., 0.2, -0.2], 0))]
fn bench_physics_step(input: StepInput) -> StepInput {
    let StepInput {
        w,
        mod_,
        tiles,
        bullet,
    } = &input;
    let collision_at = |xy: Xy| {
        let i = WorldTileIndex::from_(TileXy(xy), w);
        tiles
            .get(i.0 as usize)
            .map(|tile| vec![(ObjectsId::Tile(i), tile as &dyn Physic)])
            .unwrap_or_default()
    };
    let result: (WorldVec3, Vec<Force>, Vec<Event<ObjectsId>>) = oc_physics::step(
        black_box(w),
        black_box(1.0),
        black_box(mod_),
        (ObjectsId::Object, bullet),
        collision_at,
        |_| vec![],
        0,
        "bench",
    );
    black_box(result);
    input
}

library_benchmark_group!(
    name = bench_server_group,
    benchmarks = [
        bench_indexes_new,
        bench_visibility_compute,
        bench_physics_step
    ]
);

main!(library_benchmark_groups = bench_server_group);
