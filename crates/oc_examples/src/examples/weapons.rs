#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use bevy::prelude::*;

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use clap::{Parser, ValueEnum};
use oc_individual::order::Order;
use oc_root::{
    WorldConfig,
    battle::BattlePhase,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    side,
    utils::Frequency,
};
use oc_world::{meta::Meta, tile::Tile};
use tests::{
    individual::TestIndividual,
    squad::TestSquad,
    weapons::{TestWeapon, TestWeapons},
};

const MOD: &str = "mods/tests1";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    Stress,
    InaccuracyFastAndFar,
}

// FIXME BS NOW: ca rame plus le temps passe. Inspecter.
#[allow(unreachable_code)]
pub fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    #[cfg(not(feature = "debug"))]
    panic!("You must enable 'debug' feature for this example");

    let args = Args::parse_from(crate::args());
    let mod_ = PathBuf::from(MOD);
    let mod__ = oc_mod::Mod::load(&mod_, None)?;
    let map = PathBuf::from("maps/world1");
    let meta = Meta::from_file(&map.join("meta.toml"))?;
    let map_ = oc_world::reader::MapReader::new(&map);
    let map_ = map_.context(format!("Read map_ {}", map.display()))?;
    let w = WorldConfig::new(map_.width().unwrap() as u64, map_.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));

    let w = match args.case {
        TestCase::InaccuracyFastAndFar | TestCase::Stress => {
            w.with_individual_tick(Frequency::new(10.0))
        }
    };

    let tiles = map_.tiles(&w, &mod__).unwrap();

    let individuals = individuals(&w, &mod__, &tiles, &meta, &args);
    let squads = squads(&w, &tiles, &individuals, &args);
    let snapshot = SnapshotBuilder::new(
        map_,
        individuals,
        squads,
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .snapshot(snapshot);

    example.build().run()?;

    Ok(())
}

fn individuals(
    w: &WorldConfig,
    mod_: &oc_mod::Mod,
    _tiles: &Vec<Tile>,
    meta: &Meta,
    args: &Args,
) -> Vec<oc_individual::Individual> {
    match args.case {
        TestCase::InaccuracyFastAndFar => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(52.5, 4450., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "FullFast").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(
                    497.,
                    4510.,
                    99. * meta.geo_meters_per_z * w.geo_pixels_per_meters(),
                ))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "FullFast").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
        ],
        TestCase::Stress => {
            let mut individuals = vec![];
            for x in 0..10 {
                for y in 0..10 {
                    individuals.push(
                        TestIndividual::builder()
                            .side(side::Side::A)
                            .position(WorldVec3::new(
                                52.5 + 50. * x as f32,
                                4250. + 50. * y as f32,
                                0.,
                            ))
                            .weapons(
                                TestWeapons::builder()
                                    .primary(TestWeapon::filled(mod_, "FullFast").make())
                                    .build()
                                    .make(),
                            )
                            .build()
                            .make(w),
                    );
                }
            }
            individuals
        }
    }
}

fn squads(
    _w: &WorldConfig,
    _tiles: &Vec<Tile>,
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        TestCase::InaccuracyFastAndFar => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::Suppress(WorldVec2::new(255., 4100.))])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(1).unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(1)])
                .orders(vec![Order::Suppress(WorldVec2::new(950., 3882.))])
                .build()
                .make(),
        ],
        TestCase::Stress => {
            let mut squads = vec![];
            let mut i = 0;
            for x in 0..10 {
                for y in 0..10 {
                    squads.push(
                        TestSquad::builder()
                            .position(individuals.get(i).unwrap().position.into())
                            .members(vec![oc_individual::IndividualIndex::from(i)])
                            .orders(vec![Order::Suppress(WorldVec2::new(
                                255. + 50. * x as f32,
                                4100. + 50. * y as f32,
                            ))])
                            .build()
                            .make(),
                    );
                    i += 1;
                }
            }
            squads
        }
    }
}
