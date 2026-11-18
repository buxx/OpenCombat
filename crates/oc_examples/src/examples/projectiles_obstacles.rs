#![allow(clippy::unwrap_used)]

use std::{path::PathBuf, time::Duration};

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use bevy::prelude::*;
use clap::{Parser, ValueEnum};
use oc_battle_gui::{
    ingame::FirstIngameEnter,
    network::output::ToServerEvent,
    states::{Game, GameConfig},
};
use oc_individual::{Individual, IndividualIndex, order::Order, squad::Squad};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_root::{WorldConfig, battle::BattlePhase, geo::WorldVec3, physics::Meters, side::Side};
use oc_utils::let_some;
use oc_world::{load::WorldPath, meta::Meta, reader};
#[cfg(feature = "test")]
use oc_world_server::state::ObjectId;
use tests::{
    individual::TestIndividual,
    squad::TestSquad,
    weapons::{TestWeapon, TestWeapons},
};

use oc_projectile::spawn::SpawnProjectiles;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,

    #[arg(long, action)]
    test: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    OneAgainstWall,
    MultipleAgainstWall,
    OneAgainstHill,
    MultipleAgainstHill,
    OneTraverseVolumes,
    OneStartInsideWall,
    IndividualShotTraverseVolume,
    IndividualShotBlockedByVolume,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());
    if args.test {
        #[cfg(not(feature = "test"))]
        {
            panic!("To enable test, feature `test` must be enabled too")
        }
    }

    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None)?;
    let map_ = PathBuf::from("maps/height");
    let map = reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let world = Meta::from_file(&map_.meta());
    let world = world.context(format!("Read file {}", map_.meta().display()))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(world.geo_meters_per_z));

    #[cfg(feature = "debug")]
    if let TestCase::IndividualShotTraverseVolume | TestCase::IndividualShotBlockedByVolume =
        args.case
    {
        WorldConfig::set_inaccuracy_spread_enabled(true);
        WorldConfig::set_inaccuracy_spread(0.0);
    }

    let (individuals, squads) = individuals_and_squads(&args, &w, &mod_);
    let snapshot = SnapshotBuilder::new(
        map,
        individuals,
        squads,
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .build(w, &mod_)?;

    let example = run::Example::builder()
        .world(map_)
        .mod_(PathBuf::from("mods/tests1"))
        .install(Box::new(install))
        .snapshot(snapshot)
        .test_app_exit_code(args.test)
        .build();

    #[cfg(feature = "test")]
    let tracker = example.run().unwrap();
    #[cfg(not(feature = "test"))]
    example.run().unwrap();

    if args.test {
        #[cfg(feature = "test")]
        {
            let tracker = tracker.take();

            match args.case {
                TestCase::MultipleAgainstWall | TestCase::MultipleAgainstHill => {
                    // We consider success if physics event own at leat 10 projectiles collisions
                    let collisions = tracker
                        .physics
                        .iter()
                        .filter(|e| {
                            matches!(
                                e,
                                oc_physics::Event::Collision(
                                    ObjectId::Projectile(_),
                                    ObjectId::Tile(_),
                                    _,
                                )
                            )
                        })
                        .count();

                    assert!(collisions >= 10);
                    println!("✅ Test success");
                }
                TestCase::OneAgainstWall | TestCase::OneAgainstHill => {
                    // We consider success if physics event own at leat 10 projectiles collisions
                    let collisions = tracker
                        .physics
                        .iter()
                        .filter(|e| {
                            matches!(
                                e,
                                oc_physics::Event::Collision(
                                    ObjectId::Projectile(_),
                                    ObjectId::Tile(_),
                                    _,
                                )
                            )
                        })
                        .count();

                    assert!(collisions == 1);
                    println!("✅ Test success");
                }
                TestCase::IndividualShotTraverseVolume => {
                    // Soldier B (index 1) must be killed
                    let dead = tracker.individuals.iter().any(|event| {
                        matches!(
                            event,
                            (
                                IndividualIndex(1),
                                oc_individual::Update::SetStatus(oc_individual::Status::Dead)
                            )
                        )
                    });

                    assert!(dead);
                    println!("✅ Test success");
                }
                TestCase::IndividualShotBlockedByVolume => {
                    // Soldier A (index 0) must still be operational
                    let dead = tracker.individuals.iter().any(|event| {
                        matches!(
                            event,
                            (
                                IndividualIndex(0),
                                oc_individual::Update::SetStatus(oc_individual::Status::Dead)
                            )
                        )
                    });
                    // Ensure soldier B really shot (else test is meaningless)
                    let blocked = tracker.physics.iter().any(|e| {
                        matches!(
                            e,
                            oc_physics::Event::Collision(
                                ObjectId::Projectile(_),
                                ObjectId::Tile(_),
                                _,
                            )
                        )
                    });

                    assert!(!dead);
                    assert!(blocked);
                    println!("✅ Test success");
                }
                TestCase::OneTraverseVolumes | TestCase::OneStartInsideWall => {
                    let collisions = tracker
                        .physics
                        .iter()
                        .filter(|e| {
                            matches!(
                                e,
                                oc_physics::Event::Collision(
                                    ObjectId::Projectile(_),
                                    ObjectId::Tile(_),
                                    _,
                                )
                            )
                        })
                        .count();

                    assert!(collisions == 0);
                    println!("✅ Test success");
                }
            };
        }
    }

    Ok(())
}

fn individuals_and_squads(
    args: &Args,
    w: &WorldConfig,
    mod_: &Mod,
) -> (Vec<Individual>, Vec<Squad>) {
    match args.case {
        TestCase::IndividualShotTraverseVolume => (
            vec![
                TestIndividual::builder()
                    .side(Side::A)
                    .position(WorldVec3::new(123., 127., 0.))
                    .weapons(
                        TestWeapons::builder()
                            .primary(TestWeapon::filled(mod_, "Weapon3").make())
                            .build()
                            .make(),
                    )
                    .build()
                    .make(w),
                TestIndividual::builder()
                    .side(Side::B)
                    .position(WorldVec3::new(180., 125., 0.))
                    .build()
                    .make(w),
            ],
            vec![
                TestSquad::builder()
                    .side(Side::A)
                    .position([123., 127.].into())
                    .members(vec![IndividualIndex(0)])
                    // Wall hides soldier B: suppress his position (engage needs visibility)
                    .orders(vec![Order::Suppress([180., 125.].into())])
                    .build()
                    .make(),
                TestSquad::builder()
                    .side(Side::B)
                    .position([180., 125.].into())
                    .members(vec![IndividualIndex(1)])
                    .build()
                    .make(),
            ],
        ),
        TestCase::IndividualShotBlockedByVolume => (
            vec![
                TestIndividual::builder()
                    .side(Side::A)
                    .position(WorldVec3::new(123., 127., 0.))
                    .build()
                    .make(w),
                TestIndividual::builder()
                    .side(Side::B)
                    .position(WorldVec3::new(245., 125., 19.5))
                    .weapons(
                        TestWeapons::builder()
                            .primary(TestWeapon::filled(mod_, "Weapon3").make())
                            .build()
                            .make(),
                    )
                    .build()
                    .make(w),
            ],
            vec![
                TestSquad::builder()
                    .side(Side::A)
                    .position([123., 127.].into())
                    .members(vec![IndividualIndex(0)])
                    .build()
                    .make(),
                TestSquad::builder()
                    .side(Side::B)
                    .position([195., 125.].into())
                    .members(vec![IndividualIndex(1)])
                    // Wall hides soldier A: suppress his position (engage needs visibility)
                    .orders(vec![Order::Suppress([123., 127.].into())])
                    .build()
                    .make(),
            ],
        ),
        _ => (vec![], vec![]),
    }
}

fn install(app: &mut bevy::app::App) {
    let args = Args::parse_from(crate::args());

    if args.test {
        app.add_systems(
            Update,
            |mut commands: Commands, game: Res<Game>, g: Res<GameConfig>| {
                let_some!(g = &g.0, return);
                let timeout = match Args::parse_from(crate::args()).case {
                    TestCase::IndividualShotTraverseVolume => Duration::from_secs(15),
                    TestCase::IndividualShotBlockedByVolume => Duration::from_secs(30),
                    _ => Duration::from_secs(5),
                };
                // Ensure a minimal timeout to let time to bevy to make "network" exchange with server
                let timeout = timeout.div_f32(g.w.speed()).max(Duration::from_secs(1));
                if game.started.elapsed() > timeout {
                    commands.write_message(bevy::app::AppExit::from_code(0));
                }
            },
        );
    }

    app.add_observer(on_first_ingame_enter);
}

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    let args = Args::parse_from(crate::args());
    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None).unwrap();

    let weapon1 = mod_.weapons.iter().find(|w| w.name() == "Weapon1").unwrap();
    let weapon2 = mod_.weapons.iter().find(|w| w.name() == "Weapon2").unwrap();
    let weapon3 = mod_.weapons.iter().find(|w| w.name() == "Weapon3").unwrap();
    let ammunition = weapon1
        .ammunitions()
        .iter()
        .find(|a| a.name() == "Ammo1")
        .unwrap();
    let shot = weapon1
        .shots()
        .iter()
        .find(|s| s.name() == "Single")
        .unwrap();
    let shot3 = weapon3
        .shots()
        .iter()
        .find(|s| s.name() == "Burst3")
        .unwrap();

    match args.case {
        TestCase::MultipleAgainstHill => {
            for spawn in [
                SpawnProjectiles::new(
                    weapon1.index(),
                    ammunition.index(),
                    shot.index(),
                    10,
                    [70., 60., 8.5].into(),
                    vec![
                        (WorldVec3::new(100., 60., 8.5) - WorldVec3::new(70., 60., 8.5))
                            .normalize_or_zero();
                        10
                    ],
                    Side::A,
                    Duration::ZERO,
                    None,
                ),
                SpawnProjectiles::new(
                    weapon2.index(),
                    ammunition.index(),
                    shot.index(),
                    10,
                    [70., 65., 8.5].into(),
                    vec![
                        (WorldVec3::new(100., 65., 8.5) - WorldVec3::new(70., 65., 8.5))
                            .normalize_or_zero();
                        10
                    ],
                    Side::A,
                    Duration::ZERO,
                    None,
                ),
                SpawnProjectiles::new(
                    weapon3.index(),
                    ammunition.index(),
                    shot3.index(),
                    10,
                    [70., 70., 8.5].into(),
                    vec![
                        (WorldVec3::new(100., 70., 8.5) - WorldVec3::new(70., 70., 8.5))
                            .normalize_or_zero();
                        10
                    ],
                    Side::A,
                    Duration::ZERO,
                    None,
                ),
            ] {
                commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
            }
        }
        TestCase::OneAgainstHill => {
            let spawn = SpawnProjectiles::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                [70., 60., 8.5].into(),
                vec![
                    (WorldVec3::new(100., 60., 8.5) - WorldVec3::new(70., 60., 8.5))
                        .normalize_or_zero();
                    10
                ],
                Side::A,
                Duration::ZERO,
                None,
            );
            commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
        }
        TestCase::MultipleAgainstWall => {
            for spawn in [
                SpawnProjectiles::new(
                    weapon1.index(),
                    ammunition.index(),
                    shot.index(),
                    10,
                    [5., 125., 8.5].into(),
                    vec![
                        (WorldVec3::new(100., 125., 8.5) - WorldVec3::new(5., 125., 8.5))
                            .normalize_or_zero();
                        10
                    ],
                    Side::A,
                    Duration::ZERO,
                    None,
                ),
                SpawnProjectiles::new(
                    weapon2.index(),
                    ammunition.index(),
                    shot.index(),
                    10,
                    [5., 130., 8.5].into(),
                    vec![
                        (WorldVec3::new(100., 130., 8.5) - WorldVec3::new(5., 130., 8.5))
                            .normalize_or_zero();
                        10
                    ],
                    Side::A,
                    Duration::ZERO,
                    None,
                ),
                SpawnProjectiles::new(
                    weapon3.index(),
                    ammunition.index(),
                    shot3.index(),
                    10,
                    [5., 135., 8.5].into(),
                    vec![
                        (WorldVec3::new(100., 135., 8.5) - WorldVec3::new(5., 135., 8.5))
                            .normalize_or_zero();
                        10
                    ],
                    Side::A,
                    Duration::ZERO,
                    None,
                ),
            ] {
                commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
            }
        }
        TestCase::OneAgainstWall => {
            let spawn = SpawnProjectiles::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                [5., 125., 8.5].into(),
                vec![
                    (WorldVec3::new(100., 125., 8.5) - WorldVec3::new(5., 125., 8.5))
                        .normalize_or_zero();
                    10
                ],
                Side::A,
                Duration::ZERO,
                None,
            );
            commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
        }
        TestCase::OneTraverseVolumes => {
            let spawn = SpawnProjectiles::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                [120., 145., 1.0].into(),
                vec![
                    (WorldVec3::new(120., 245., 1.0) - WorldVec3::new(120., 145., 1.0))
                        .normalize_or_zero();
                    10
                ],
                Side::A,
                Duration::ZERO,
                None,
            );
            commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
        }
        TestCase::IndividualShotTraverseVolume | TestCase::IndividualShotBlockedByVolume => {}
        TestCase::OneStartInsideWall => {
            let spawn = SpawnProjectiles::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                [122., 136., 8.5].into(),
                vec![
                    (WorldVec3::new(122., 236., 8.5) - WorldVec3::new(122., 136., 8.5))
                        .normalize_or_zero();
                    10
                ],
                Side::A,
                Duration::ZERO,
                None,
            );
            commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
        }
    }
}
