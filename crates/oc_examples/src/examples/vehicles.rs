#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use clap::{Parser, ValueEnum};
use oc_geo::tile::WorldTileIndex;
use oc_individual::{IndividualIndex, order::Order};
use oc_mod::{Mod, vehicle::CrewRole};
use oc_root::{
    WcfgFrom, WorldConfig, battle::BattlePhase, geo::WorldVec2, physics::Meters, side::Side,
};
use oc_utils::{d2::Direction, index::AtIndex};
use oc_vehicle::{Vehicle, VehicleIndex};
use oc_world::{meta::Meta, tile::Tile};
use tests::{individual::TestIndividual, squad::TestSquad};

const MOD: &str = "mods/tests1";
const MAP: &str = "maps/meadow1";
const TANK: &str = "Tank1";

const TMF_START: [f32; 2] = [150., 250.];
const TMF_TARGET: [f32; 2] = [150., 150.];

const TT_START: [f32; 2] = [150., 250.];
const TT_TARGET: [f32; 2] = [150., 150.];
const TT_TOLERANCE: f32 = 0.01;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,

    /// Exit with success when the case expectation is reached, error on timeout
    #[arg(long, action)]
    test: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    TankMove,
    TankTarget,
    DrawTanks,
}

struct TankSetup {
    side: Side,
    position: WorldVec2,
    chassis: Direction,
    turret: Option<Direction>,
    crew: Vec<CrewRole>,
    reload_progress: f32,
}

impl TankSetup {
    fn new(side: Side, position: [f32; 2], chassis: Direction, turret: Option<Direction>) -> Self {
        Self {
            side,
            position: WorldVec2::new(position[0], position[1]),
            chassis,
            turret,
            crew: vec![CrewRole::Driver],
            reload_progress: oc_vehicle::LOADED,
        }
    }

    fn with_reload_progress(mut self, value: f32) -> Self {
        self.reload_progress = value;
        self
    }

    fn with_crew(mut self, crew: Vec<CrewRole>) -> Self {
        self.crew = crew;
        self
    }
}

// FIXME BS NOW : tests and code to implement:
// - projectile collision with vehicle (all chassis surface, when oriented too)
// - Tank destruction (penetration) and weak parts
// - Tank fire/reload/ammunitions (with chassis/turret direction)
// - Tank chassis visible (when crew hidden position)
pub fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());

    #[cfg(not(feature = "test"))]
    if args.test {
        panic!("Execute with --test need `test` feature enabled")
    }

    let mod_ = PathBuf::from(MOD);
    let mod__ = oc_mod::Mod::load(&mod_, None)?;
    let map = PathBuf::from(MAP);
    let meta = Meta::from_file(&map.join("meta.toml"))?;
    let map_ = oc_world::reader::MapReader::new(&map);
    let map_ = map_.context(format!("Read map_ {}", map.display()))?;
    let w = WorldConfig::new(map_.width().unwrap() as u64, map_.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    let tiles = map_.tiles(&w, &mod__).unwrap();

    let tanks = tanks(&args);
    let individuals = individuals(&w, &tiles, &tanks, &args);
    let squads = squads(&w, &tiles, &individuals, &args);
    let vehicles = vehicles(&w, &tiles, &mod__, &tanks);
    let snapshot = SnapshotBuilder::new(
        map_,
        individuals,
        squads,
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .with_vehicles(vehicles)
    .build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .test_app_exit_code(args.test)
        .snapshot(snapshot);

    #[cfg(feature = "test")]
    let example = example.install({
        let args = args.clone();
        Box::new(move |app: &mut bevy::app::App| {
            if args.test {
                app.insert_resource(test::Args_(args.clone()))
                    .add_systems(bevy::app::Update, test::end_when_success_or_timeout);
            }
        })
    });

    example.build().run()?;

    Ok(())
}

fn tanks(args: &Args) -> Vec<TankSetup> {
    let diagonal = Direction::new(1., 1.);
    match args.case {
        // Tank facing est, ordered to move north: rotates its chassis then moves. Gun not loaded.
        TestCase::TankMove => vec![
            TankSetup::new(Side::A, TMF_START, Direction::EST, Some(Direction::EST))
                .with_crew(vec![
                    CrewRole::Commander,
                    CrewRole::Driver,
                    CrewRole::Gunner,
                    CrewRole::Loader,
                ])
                .with_reload_progress(0.),
        ],
        TestCase::TankTarget => vec![
            TankSetup::new(Side::A, TT_START, Direction::EST, Some(Direction::EST)).with_crew(
                vec![
                    CrewRole::Commander,
                    CrewRole::Driver,
                    CrewRole::Gunner,
                    CrewRole::Loader,
                ],
            ),
        ],
        TestCase::DrawTanks => vec![
            // Turret aligned with chassis, full crew
            TankSetup::new(
                Side::A,
                [150., 250.],
                Direction::NORTH,
                Some(Direction::NORTH),
            )
            .with_crew(vec![
                CrewRole::Commander,
                CrewRole::Gunner,
                CrewRole::Driver,
                CrewRole::Operator,
            ]),
            // Turret rotated relatively to chassis
            TankSetup::new(
                Side::A,
                [200., 250.],
                Direction::EST,
                Some(Direction::NORTH),
            ),
            TankSetup::new(Side::A, [250., 250.], diagonal, Some(Direction::WEST)),
            // Without turret
            TankSetup::new(Side::A, [300., 250.], Direction::SOUTH, None),
            // Enemy: visible when its crew is visible
            TankSetup::new(Side::B, [225., 190.], Direction::SOUTH, Some(diagonal)),
        ],
    }
}

fn z(w: &WorldConfig, tiles: &[Tile], position: WorldVec2) -> f32 {
    tiles.at(WorldTileIndex::from_(position, w)).z_pixels(w)
}

fn individuals(
    w: &WorldConfig,
    tiles: &[Tile],
    tanks: &[TankSetup],
    args: &Args,
) -> Vec<oc_individual::Individual> {
    match args.case {
        TestCase::TankMove | TestCase::TankTarget | TestCase::DrawTanks => crews(w, tiles, tanks),
    }
}

fn crews(w: &WorldConfig, tiles: &[Tile], tanks: &[TankSetup]) -> Vec<oc_individual::Individual> {
    let mut individuals = vec![];

    for (i, tank) in tanks.iter().enumerate() {
        let position = tank.position.extend(z(w, tiles, tank.position));
        for role in &tank.crew {
            let individual = TestIndividual::builder()
                .side(tank.side)
                .position(position)
                .build()
                .make(w)
                .with_embarked(Some((VehicleIndex(i as u64), *role)));
            individuals.push(individual);
        }
    }

    individuals
}

fn squads(
    _w: &WorldConfig,
    _tiles: &[Tile],
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        TestCase::TankMove => crew_squads(individuals, vec![Order::MoveTo(TMF_TARGET.into())]),
        TestCase::TankTarget => crew_squads(individuals, vec![Order::Suppress(TT_TARGET.into())]),
        TestCase::DrawTanks => crew_squads(individuals, vec![]),
    }
}

fn crew_squads(
    individuals: &[oc_individual::Individual],
    orders: Vec<Order>,
) -> Vec<oc_individual::squad::Squad> {
    let mut squads: Vec<(Option<VehicleIndex>, Vec<IndividualIndex>)> = vec![];

    for (i, individual) in individuals.iter().enumerate() {
        let vehicle = individual.embarked.map(|(vehicle, _)| vehicle);
        match squads.last_mut() {
            Some((vehicle_, members)) if vehicle.is_some() && *vehicle_ == vehicle => {
                members.push(IndividualIndex(i as u64));
            }
            _ => squads.push((vehicle, vec![IndividualIndex(i as u64)])),
        }
    }

    squads
        .into_iter()
        .map(|(_, members)| {
            let leader = &individuals[members[0].0 as usize];
            TestSquad::builder()
                .side(leader.side)
                .position(leader.position.into())
                .members(members)
                .orders(orders.clone())
                .build()
                .make()
        })
        .collect()
}

fn vehicles(w: &WorldConfig, tiles: &[Tile], mod_: &Mod, tanks: &[TankSetup]) -> Vec<Vehicle> {
    let type_ = mod_.vehicle_from_name(TANK).unwrap().index();

    tanks
        .iter()
        .map(|tank| {
            Vehicle::new(
                tank.side,
                type_,
                oc_mod::illustration::IllustrationIndex(0),
                tank.position.extend(z(w, tiles, tank.position)),
                vec![],
                tank.chassis,
                tank.turret,
            )
            .with_reload_progress(tank.reload_progress)
        })
        .collect()
}

#[cfg(feature = "test")]
mod test {
    use std::{
        sync::Mutex,
        time::{Duration, Instant},
    };

    use bevy::prelude::*;
    use oc_battle_gui::{states::Game, world::World};
    use oc_root::Wcfg;
    use oc_root::geo::WorldVec2;
    use oc_utils::d2::Direction;
    use oc_utils::let_some;
    use oc_vehicle::VehicleIndex;

    use super::{Args, TMF_TARGET, TT_TARGET, TT_TOLERANCE, TestCase};

    #[derive(Debug, Resource)]
    pub struct Args_(pub Args);

    /// Exit with success code when case expectation is reached (and kept during one
    /// second), with error code when timeout is reached.
    pub fn end_when_success_or_timeout(
        mut commands: Commands,
        game: Res<Game>,
        args: Res<Args_>,
        w: Res<Wcfg>,
        world: Res<World>,
    ) {
        let_some!(w = &w.0, return);
        static SUCCESS_SINCE: Mutex<Option<Instant>> = Mutex::new(None);

        let speed = w.speed();
        let timeout = match args.0.case {
            TestCase::TankMove => Duration::from_secs(60),
            TestCase::TankTarget => Duration::from_secs(30),
            TestCase::DrawTanks => Duration::from_secs(5),
        };
        let timeout = timeout.div_f32(speed).max(Duration::from_secs(5));

        let success = match args.0.case {
            TestCase::TankMove => world
                .vehicle(VehicleIndex(0))
                .map(|vehicle| {
                    let (dx, dy) = (
                        vehicle.position.x - TMF_TARGET[0],
                        vehicle.position.y - TMF_TARGET[1],
                    );
                    (dx * dx + dy * dy).sqrt() <= oc_vehicle::DESTINATION_TOLERANCE
                })
                .unwrap_or_default(),
            TestCase::TankTarget => world
                .vehicle(VehicleIndex(0))
                .and_then(|vehicle| {
                    let from: Vec2 = WorldVec2::from(vehicle.position).into();
                    let target = Direction::from_points2d(from, TT_TARGET.into());
                    vehicle
                        .turret_direction
                        .map(|turret| turret.angle_to(&target).abs() <= TT_TOLERANCE)
                })
                .unwrap_or_default(),
            TestCase::DrawTanks => true,
        };

        let mut since = SUCCESS_SINCE.lock().unwrap();
        *since = match (success, *since) {
            (true, None) => Some(Instant::now()),
            (true, Some(since)) => Some(since),
            (false, _) => None,
        };

        if since.is_some_and(|since| since.elapsed().as_secs_f32() > 1. / speed) {
            println!("✅ SUCCESS ! ({:?})", args.0.case);
            commands.write_message(bevy::app::AppExit::from_code(0));
        } else if game.started.elapsed() > timeout {
            eprintln!("❌ FAILED ! Timeout reached ({:?})", args.0.case);
            commands.write_message(bevy::app::AppExit::from_code(1));
        }
    }
}
