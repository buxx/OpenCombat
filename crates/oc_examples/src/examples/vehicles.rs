#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use crate::{
    logging, run,
    snapshot::SnapshotBuilder,
    utils::{self, Firing, Loaded},
};
use clap::{Parser, ValueEnum};
use oc_battle_gui::{ingame::FirstIngameEnter, network::output::ToServerEvent};
use oc_individual::{IndividualIndex, order::Order};
use oc_mod::{Mod, vehicle::CrewRole};
use oc_network::ToServer;
use oc_projectile::spawn::SpawnProjectiles;
use oc_root::{
    WorldConfig,
    geo::{WorldVec2, WorldVec3},
    side::Side,
};
use oc_utils::d2::Direction;
use oc_vehicle::{Vehicle, VehicleIndex};
use oc_world::tile::Tile;
use tests::squad::TestSquad;

const MOD: &str = "mods/tests1";
const MAP: &str = "maps/meadow1";
const TANK: &str = "Tank1";

const TMF_START: [f32; 2] = [150., 250.];
const TMF_TARGET: [f32; 2] = [250., 250.];
const TMR_TARGET: [f32; 2] = [150., 150.];

const TT_START: [f32; 2] = [150., 250.];
const TT_TARGET: [f32; 2] = [150., 150.];
const TT_TOLERANCE: f32 = 0.01;

// Target far enough (30m) to keep the tank crew out of the shell shrapnel rayon (25m)
const TF_START: [f32; 2] = [150., 250.];
const TF_TARGET: [f32; 2] = [150., 100.];
/// Explosion must be spawned at this distance (pixels) of the target
const TF_TOLERANCE: f32 = 1.;
const TF_SHELL: &str = "5/38 shell";
const TF_SHELLS: u32 = 10;

// Tank1 chassis (6.5m x 2.7m, height 2.5m, 5 pixels per meter) facing est at `TP_TANK` covers
// x 133.75..166.25, y 243.25..256.75 and z ground..ground+12.5 (pixels).
const TP_TANK: [f32; 2] = [150., 250.];
// Projectiles start south of the tank (far enough to watch them move) and fly north.
const TP_FROM_Y: f32 = 310.;
const TP_DIRECTION: [f32; 3] = [0., -1., 0.];
/// Pass just beside the chassis rear end (1.75 pixels, 0.35m): no collision
const TP_NEAR: ([f32; 2], f32) = ([168., TP_FROM_Y], 6.);
/// Cross the chassis center at mid height: collision
const TP_HIT: ([f32; 2], f32) = ([150., TP_FROM_Y], 6.);
/// Cross over the chassis center, above its height: no collision
const TP_OVER: ([f32; 2], f32) = ([150., TP_FROM_Y], 15.);

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
    TankMoveForward,
    TankMoveRotation,
    TankTarget,
    TankFire,
    TankProjectiles,
    DrawTanks,
}

struct TankSetup {
    side: Side,
    position: WorldVec2,
    chassis: Direction,
    turret: Option<Direction>,
    crew: Vec<CrewRole>,
    reload_progress: f32,
    shells: u32,
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
            shells: 0,
        }
    }

    fn with_reload_progress(mut self, value: f32) -> Self {
        self.reload_progress = value;
        self
    }

    fn with_shells(mut self, value: u32) -> Self {
        self.shells = value;
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
// - Tank move during "natural" target (chassis must move, gunner mst adjust turrent direction)
pub fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());

    utils::ensure_test_feature(args.test);

    let mod_ = PathBuf::from(MOD);
    let map = PathBuf::from(MAP);
    let loaded = Loaded::new(&mod_, &map)?;
    let (w, mod__) = (&loaded.w, &loaded.mod_);
    let tiles = loaded.tiles(w)?;

    let tanks = tanks(&args);
    let individuals = crews(w, &tiles, &tanks);
    let squads = squads(&individuals, &args);
    let vehicles = vehicles(w, &tiles, mod__, &tanks);
    let projectiles = projectiles(w, &tiles, mod__, &args);
    let snapshot = SnapshotBuilder::fight(loaded.map.clone(), individuals, squads)
        .with_vehicles(vehicles)
        .build(w.clone(), mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .test_app_exit_code(args.test)
        .snapshot(snapshot);

    let example = example.install({
        let args = args.clone();
        Box::new(move |app: &mut bevy::app::App| {
            if !projectiles.is_empty() {
                let projectiles = projectiles.clone();
                app.add_observer(
                    move |_: bevy::prelude::On<FirstIngameEnter>,
                          mut commands: bevy::prelude::Commands| {
                        for spawn in projectiles.clone() {
                            commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
                        }
                    },
                );
            }

            #[cfg(feature = "test")]
            if args.test {
                app.insert_resource(test::Args_(args.clone()))
                    .add_observer(test::on_fx)
                    .add_systems(bevy::app::Update, test::end_when_success_or_timeout);
            }
            #[cfg(not(feature = "test"))]
            let _ = &args;
        })
    });

    #[cfg(feature = "test")]
    {
        let tracker = example.build().run()?;
        if args.test {
            test::check_tracker(&args, &tracker.take());
        }
    }
    #[cfg(not(feature = "test"))]
    example.build().run()?;

    Ok(())
}

fn tanks(args: &Args) -> Vec<TankSetup> {
    let diagonal = Direction::new(1., 1.);
    match args.case {
        // Tank facing est, ordered to move north: rotates its chassis then moves. Gun not loaded.
        TestCase::TankMoveForward => vec![
            TankSetup::new(Side::A, TMF_START, Direction::EST, Some(Direction::EST))
                .with_crew(vec![
                    CrewRole::Commander,
                    CrewRole::Driver,
                    CrewRole::Gunner,
                    CrewRole::Loader,
                ])
                .with_reload_progress(0.),
        ],
        // Tank facing est, ordered to move north: rotates its chassis then moves. Gun not loaded.
        TestCase::TankMoveRotation => vec![
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
        // Tank facing est, ordered to suppress north: rotates its turret then fires
        TestCase::TankFire => vec![
            TankSetup::new(Side::A, TF_START, Direction::EST, Some(Direction::EST))
                .with_crew(vec![
                    CrewRole::Commander,
                    CrewRole::Driver,
                    CrewRole::Gunner,
                    CrewRole::Loader,
                ])
                .with_shells(TF_SHELLS),
        ],
        TestCase::TankProjectiles => vec![TankSetup::new(
            Side::A,
            TP_TANK,
            Direction::EST,
            Some(Direction::EST),
        )],
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

fn crews(w: &WorldConfig, tiles: &[Tile], tanks: &[TankSetup]) -> Vec<oc_individual::Individual> {
    let mut individuals = vec![];

    for (i, tank) in tanks.iter().enumerate() {
        let position = utils::on_ground(w, tiles, tank.position);
        for role in &tank.crew {
            let individual = utils::soldier(w, tank.side, position)
                .with_embarked(Some((VehicleIndex(i as u64), *role)));
            individuals.push(individual);
        }
    }

    individuals
}

fn squads(
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        TestCase::TankMoveForward => {
            crew_squads(individuals, vec![Order::MoveTo(TMF_TARGET.into())])
        }
        TestCase::TankMoveRotation => {
            crew_squads(individuals, vec![Order::MoveTo(TMR_TARGET.into())])
        }
        TestCase::TankTarget => crew_squads(individuals, vec![Order::Suppress(TT_TARGET.into())]),
        TestCase::TankFire => crew_squads(individuals, vec![Order::Suppress(TF_TARGET.into())]),
        TestCase::TankProjectiles | TestCase::DrawTanks => crew_squads(individuals, vec![]),
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
    let shell = mod_.shell_from_name(TF_SHELL).unwrap().index();

    tanks
        .iter()
        .map(|tank| {
            Vehicle::new(
                tank.side,
                type_,
                oc_mod::illustration::IllustrationIndex(0),
                utils::on_ground(w, tiles, tank.position),
                vec![],
                tank.chassis,
                tank.turret,
            )
            .with_reload_progress(tank.reload_progress)
            .with_shells(vec![(shell, tank.shells)])
        })
        .collect()
}

/// Projectiles sent to the server at game start. Spawned in this order, so their ids are
/// `ProjectileId(0)`, `ProjectileId(1)`, ...
fn projectiles(w: &WorldConfig, tiles: &[Tile], mod_: &Mod, args: &Args) -> Vec<SpawnProjectiles> {
    match args.case {
        TestCase::TankProjectiles => {
            let firing = Firing::weapon1(mod_);

            [TP_NEAR, TP_HIT, TP_OVER]
                .into_iter()
                .map(|(from, height)| {
                    let mut from = utils::on_ground(w, tiles, from.into());
                    from.z += height;
                    firing.spawn(1, from, vec![WorldVec3::from(TP_DIRECTION)], Side::B)
                })
                .collect()
        }
        TestCase::TankMoveForward
        | TestCase::TankMoveRotation
        | TestCase::TankTarget
        | TestCase::TankFire
        | TestCase::DrawTanks => vec![],
    }
}

#[cfg(feature = "test")]
mod test {
    use std::{
        sync::Mutex,
        time::{Duration, Instant},
    };

    use bevy::prelude::*;
    use oc_battle_gui::{states::Game, world::World};
    use oc_individual::squad::SquadIndex;
    use oc_root::Wcfg;
    use oc_root::geo::WorldVec2;
    use oc_utils::d2::Direction;
    use oc_utils::let_some;
    use oc_vehicle::VehicleIndex;

    use oc_projectile::ProjectileId;
    use oc_world_server::{state::ObjectId, tracker::Tracker_};

    use crate::examples::vehicles::TMR_TARGET;

    use super::{Args, TF_TARGET, TF_TOLERANCE, TMF_TARGET, TT_TARGET, TT_TOLERANCE, TestCase};

    /// Set when an explosion is spawned at `TF_TARGET`
    static TANK_FIRE_EXPLOSION: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);

    /// Record explosion spawned at the tank fire target
    pub fn on_fx(fx: On<oc_battle_gui::fx::FxEvent>) {
        use oc_physics::fx::{Fx, Visual};

        let Fx::Visual(Visual::Explosion(position, _)) = &fx.0 else {
            return;
        };
        let position: Vec2 = (*position).into();
        if position.distance(TF_TARGET.into()) <= TF_TOLERANCE {
            TANK_FIRE_EXPLOSION.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

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
            TestCase::TankMoveForward | TestCase::TankMoveRotation => Duration::from_secs(60),
            TestCase::TankTarget => Duration::from_secs(30),
            // Turret rotation (90° at 45°/s) then shell flight
            TestCase::TankFire => Duration::from_secs(30),
            // Let projectiles cross the tank (~3s at Weapon1 velocity)
            TestCase::TankProjectiles => Duration::from_secs(6),
            TestCase::DrawTanks => Duration::from_secs(5),
        };
        let timeout = timeout.div_f32(speed).max(Duration::from_secs(5));

        let success = match args.0.case {
            TestCase::TankMoveForward => move_accomplished(&world, TMF_TARGET),
            TestCase::TankMoveRotation => move_accomplished(&world, TMR_TARGET),
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
            TestCase::TankFire => TANK_FIRE_EXPLOSION.load(std::sync::atomic::Ordering::Relaxed),
            TestCase::DrawTanks => true,
            // Checked after app exit, from server tracker (see `check_tracker`)
            TestCase::TankProjectiles => false,
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
        } else if game.started.elapsed() > timeout
            && let TestCase::TankProjectiles = args.0.case
        {
            commands.write_message(bevy::app::AppExit::from_code(0));
        } else if game.started.elapsed() > timeout {
            eprintln!("❌ FAILED ! Timeout reached ({:?})", args.0.case);
            commands.write_message(bevy::app::AppExit::from_code(1));
        }
    }

    /// Tank reached `target` and its move order is finished: squad and crew have no more orders
    fn move_accomplished(world: &World, target: [f32; 2]) -> bool {
        let reached = world
            .vehicle(VehicleIndex(0))
            .map(|vehicle| {
                let position: Vec2 = WorldVec2::from(vehicle.position).into();
                position.distance(target.into()) <= oc_vehicle::DESTINATION_TOLERANCE
            })
            .unwrap_or_default();
        let_some!(squad = world.squad(SquadIndex(0)), return false);
        let crew_without_orders = squad.members.iter().all(|member| {
            world
                .individual(*member)
                .is_some_and(|individual| individual.orders.is_empty())
        });

        reached && squad.orders.is_empty() && crew_without_orders
    }

    /// Check, after app exit, expectations based on server tracker. Panic if not satisfied.
    pub fn check_tracker(args: &Args, tracker: &Tracker_) {
        match args.case {
            TestCase::TankProjectiles => {
                let collided = |projectile: ProjectileId| {
                    tracker.physics.iter().any(|event| match event {
                        oc_physics::Event::Collision(a, b, _) => matches!(
                            (a, b),
                            (ObjectId::Projectile(p), ObjectId::Vehicle(VehicleIndex(0)))
                            | (ObjectId::Vehicle(VehicleIndex(0)), ObjectId::Projectile(p))
                            if *p == projectile
                        ),
                        _ => false,
                    })
                };

                // Ids follow spawn order, see `super::projectiles`
                let expectations = [
                    ("near (beside chassis)", ProjectileId(0), false),
                    ("hit (through chassis)", ProjectileId(1), true),
                    ("over (above chassis)", ProjectileId(2), false),
                ];
                let mut success = true;
                for (name, projectile, expected) in expectations {
                    let collided = collided(projectile);
                    let ok = collided == expected;
                    success &= ok;
                    println!(
                        "{} projectile {name}: collided={collided}, expected={expected}",
                        if ok { "✅" } else { "❌" }
                    );
                }

                if !success {
                    eprintln!("❌ FAILED ! ({:?})", args.case);
                    std::process::exit(1);
                }
                println!("✅ SUCCESS ! ({:?})", args.case);
            }
            TestCase::TankMoveForward
            | TestCase::TankMoveRotation
            | TestCase::TankTarget
            | TestCase::TankFire
            | TestCase::DrawTanks => {}
        }
    }
}
