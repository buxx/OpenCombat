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
/// Same as `TANK`, but penetrated by `TF_SHELL`
const LIGHT_TANK: &str = "LightTank1";

const TMF_START: [f32; 2] = [150., 250.];
const TMF_TARGET: [f32; 2] = [250., 250.];
const TMR_TARGET: [f32; 2] = [150., 150.];

// Tank crosses the map from south to north (80m) while an enemy, out of the tank path and of the
// shell shrapnel rayon (25m), is visible. Gunner must target and kill it during the move.
const TMDT_START: [f32; 2] = [150., 450.];
const TMDT_TARGET: [f32; 2] = [150., 50.];
// 40m from the tank path
const TMDT_ENEMY: [f32; 2] = [350., 250.];
/// Enemy index: after the tank crew (commander, driver, gunner, loader)
#[cfg(feature = "test")]
const TMDT_ENEMY_INDEX: u64 = 4;

const TT_START: [f32; 2] = [150., 250.];
const TT_TARGET: [f32; 2] = [150., 150.];
#[cfg(feature = "test")]
const TT_TOLERANCE: f32 = 0.01;

// Target far enough (30m) to keep the tank crew out of the shell shrapnel rayon (25m)
const TF_START: [f32; 2] = [150., 250.];
const TF_TARGET: [f32; 2] = [150., 100.];
/// Explosion must be spawned at this distance (pixels) of the target
#[cfg(feature = "test")]
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
// Fired projectile hit wall
const TPHO_START: [f32; 2] = [32.5, 250.];
const TPHO_TARGET: [f32; 2] = [32.5, 30.];
// Tank chassis visible: meadow1 BrickWall square covers x 15..50, y 60..95 (pixels). Watcher stands
// south of it. Enemy tank (facing est) stands north of it: its position (and its crew) is hidden by
// the walls, but its chassis rear end (x 38.75..71.25) exceeds the walls east side.
const TCV_WATCHER: [f32; 2] = [32.5, 150.];
const TCV_TANK: [f32; 2] = [55., 50.];
// Tank crew stunned/killed: a `TF_SHELL` (penetration force 1.0) explodes on the chassis of a
// tank. `TANK` resists to the penetration (crew is stunned), `LIGHT_TANK` does not (crew is killed).
// "By explosion": explosion is directly spawned on the chassis of a lone tank (at `VH_TANK`).
// "By tank": attacker tank (side B, at `VH_ATTACKER`, 30m south of the target tank) is ordered to
// fire one shell on the target tank: shell explodes on the target chassis south side.
const VH_TANK: [f32; 2] = [150., 100.];
const VH_ATTACKER: [f32; 2] = [150., 250.];
/// Target tank crew (commander, driver, gunner, loader) indexes: target tank is the first one
#[cfg(feature = "test")]
const VH_CREW: [u64; 4] = [0, 1, 2, 3];
/// Expected crew stun duration (game seconds) when the shell does not penetrate the chassis
#[cfg(feature = "test")]
const VH_STUN_DURATION: f32 = 30.;
/// Tolerance (game seconds) on stun duration (individual tick is one second)
#[cfg(feature = "test")]
const VH_STUN_TOLERANCE: f32 = 3.;
// Tank lov: tank chassis (crewless, facing north: its length is along y axis) stands between a
// watcher (side A, west) and a hidden enemy (side B, est). Control enemy (side B) is not behind the
// chassis. Hidden enemy must stay not visible while control one is visible.
const TL_TANK: [f32; 2] = [150., 250.];
const TL_WATCHER: [f32; 2] = [100., 250.];
const TL_HIDDEN: [f32; 2] = [180., 250.];
const TL_CONTROL: [f32; 2] = [180., 310.];
/// Watcher, hidden and control indexes (tank has no crew)
#[cfg(feature = "test")]
const TL_HIDDEN_INDEX: u64 = 1;
#[cfg(feature = "test")]
const TL_CONTROL_INDEX: u64 = 2;
/// Obstacle tiles nature
#[cfg(feature = "test")]
const BRICKWALL: &str = "BrickWall";

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
    TankMoveDuringTarget,
    TankTarget,
    TankFire,
    TankProjectiles,
    TankProjectileHitObject,
    TankChassisVisible,
    TankLov,
    TankCrewStunnedByExplosion,
    TankCrewKilledByExplosion,
    TankCrewStunnedByTank,
    TankCrewKilledByTank,
    DrawTanks,
}

struct TankSetup {
    type_: &'static str,
    side: Side,
    position: WorldVec2,
    chassis: Direction,
    turret: Option<Direction>,
    crew: Vec<CrewRole>,
    reload_progress: f32,
    shells: u32,
    /// Crew squad receives the case orders
    ordered: bool,
}

impl TankSetup {
    fn new(side: Side, position: [f32; 2], chassis: Direction, turret: Option<Direction>) -> Self {
        Self {
            type_: TANK,
            side,
            position: WorldVec2::new(position[0], position[1]),
            chassis,
            turret,
            crew: vec![CrewRole::Driver],
            reload_progress: oc_vehicle::LOADED,
            shells: 0,
            ordered: true,
        }
    }

    fn with_type(mut self, value: &'static str) -> Self {
        self.type_ = value;
        self
    }

    fn without_orders(mut self) -> Self {
        self.ordered = false;
        self
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
//    ---> FIXME: add random with more or less around penetration score to permit luck)
// - Tank fire/reload/ammunitions (with chassis/turret direction)
// - Tank chassis visible (when crew hidden position)
// - Tank move during "natural" target (chassis must move, gunner mst adjust turrent direction):
//   see `TestCase::TankMoveDuringTarget`
pub fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());

    utils::ensure_test_feature(args.test);
    #[cfg(not(feature = "debug"))]
    if !explosions(&args).is_empty() {
        anyhow::bail!("{:?} case requires the `debug` feature", args.case);
    }

    let mod_ = PathBuf::from(MOD);
    let map = PathBuf::from(MAP);
    let loaded = Loaded::new(&mod_, &map)?;
    let w = &world_config(loaded.w.clone(), &args);
    let mod__ = &loaded.mod_;
    let tiles = loaded.tiles(w)?;

    let tanks = tanks(&args);
    let mut individuals = crews(w, &tiles, &tanks);
    individuals.extend(enemies(w, &tiles, &args));
    let squads = squads(&individuals, &tanks, &args);
    let vehicles = vehicles(w, &tiles, mod__, &tanks);
    let projectiles = projectiles(w, &tiles, mod__, &args);
    #[cfg(feature = "debug")]
    let (explosions, shell) = (
        explosions(&args),
        mod__.shell_from_name(TF_SHELL).unwrap().index(),
    );
    #[cfg(feature = "test")]
    let obstacles = obstacles(&tiles, mod__);
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

            #[cfg(feature = "debug")]
            if !explosions.is_empty() {
                let explosions = explosions.clone();
                app.add_observer(
                    move |_: bevy::prelude::On<FirstIngameEnter>,
                          mut commands: bevy::prelude::Commands| {
                        for position in explosions.clone() {
                            let explosion = ToServer::DebugExplosion(shell, position);
                            commands.trigger(ToServerEvent(explosion));
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
            test::check_tracker(&args, &obstacles, &tracker.take());
        }
    }
    #[cfg(not(feature = "test"))]
    example.build().run()?;

    Ok(())
}

// FIXME BS NOW: mesure stun server side to permit speed test
/// Max speed (OC_SPEED) for crew stun cases: stun duration is measured client side in real time,
/// a higher speed makes the tolerance (3 game seconds) shorter than client update latency.
#[cfg(feature = "debug")]
const VH_STUN_MAX_SPEED: f32 = 5.;

fn world_config(w: WorldConfig, args: &Args) -> WorldConfig {
    #[cfg(feature = "debug")]
    if matches!(
        args.case,
        TestCase::TankCrewStunnedByExplosion | TestCase::TankCrewStunnedByTank
    ) && w.speed() > VH_STUN_MAX_SPEED
    {
        WorldConfig::set_speed((VH_STUN_MAX_SPEED * 10.) as u32);
    }

    match args.case {
        // Target tank crew is the whole side A: once stunned, side A morale is zero. Prevent battle
        // end by morale, else the battle ends before the stun ends.
        TestCase::TankCrewStunnedByTank => w.with_morale_end_threshold(0.),
        TestCase::TankMoveForward
        | TestCase::TankMoveRotation
        | TestCase::TankMoveDuringTarget
        | TestCase::TankTarget
        | TestCase::TankFire
        | TestCase::TankProjectiles
        | TestCase::TankProjectileHitObject
        | TestCase::TankChassisVisible
        | TestCase::TankLov
        | TestCase::TankCrewStunnedByExplosion
        | TestCase::TankCrewKilledByExplosion
        | TestCase::TankCrewKilledByTank
        | TestCase::DrawTanks => w,
    }
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
        // Tank facing north, ordered to move far north, enemy visible at est of its path
        TestCase::TankMoveDuringTarget => vec![
            TankSetup::new(
                Side::A,
                TMDT_START,
                Direction::NORTH,
                Some(Direction::NORTH),
            )
            .with_crew(vec![
                CrewRole::Commander,
                CrewRole::Driver,
                CrewRole::Gunner,
                CrewRole::Loader,
            ])
            .with_shells(TF_SHELLS),
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
        // Tank facing north, ordered to suppress behind the BrickWall square: shell hits the wall
        TestCase::TankProjectileHitObject => vec![
            TankSetup::new(
                Side::A,
                TPHO_START,
                Direction::NORTH,
                Some(Direction::NORTH),
            )
            .with_crew(vec![
                CrewRole::Commander,
                CrewRole::Driver,
                CrewRole::Gunner,
                CrewRole::Loader,
            ])
            .with_shells(1),
        ],
        // Enemy tank behind the BrickWall square, its chassis exceeding the walls
        TestCase::TankChassisVisible => vec![TankSetup::new(
            Side::B,
            TCV_TANK,
            Direction::EST,
            Some(Direction::EST),
        )],
        // Crewless tank between watcher and hidden enemy
        TestCase::TankLov => vec![
            TankSetup::new(Side::A, TL_TANK, Direction::NORTH, Some(Direction::NORTH))
                .with_crew(vec![])
                .without_orders(),
        ],
        // Lone tank, a shell explodes on its chassis
        TestCase::TankCrewStunnedByExplosion => vec![shelled_tank(TANK)],
        TestCase::TankCrewKilledByExplosion => vec![shelled_tank(LIGHT_TANK)],
        // Target tank first, then the attacker
        TestCase::TankCrewStunnedByTank => vec![shelled_tank(TANK), attacker_tank()],
        TestCase::TankCrewKilledByTank => vec![shelled_tank(LIGHT_TANK), attacker_tank()],
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
            // Enemy: visible when its chassis is visible
            TankSetup::new(Side::B, [225., 190.], Direction::SOUTH, Some(diagonal)),
        ],
    }
}

/// Target tank of `type_` with full crew, without shells nor orders: a shell explodes on its chassis
fn shelled_tank(type_: &'static str) -> TankSetup {
    TankSetup::new(Side::A, VH_TANK, Direction::EST, Some(Direction::EST))
        .with_type(type_)
        .with_crew(vec![
            CrewRole::Commander,
            CrewRole::Driver,
            CrewRole::Gunner,
            CrewRole::Loader,
        ])
        .without_orders()
}

/// Enemy tank, facing the target tank, with one shell to fire on it
fn attacker_tank() -> TankSetup {
    TankSetup::new(
        Side::B,
        VH_ATTACKER,
        Direction::NORTH,
        Some(Direction::NORTH),
    )
    .with_crew(vec![
        CrewRole::Commander,
        CrewRole::Driver,
        CrewRole::Gunner,
        CrewRole::Loader,
    ])
    .with_shells(1)
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

/// Not embarked individuals, enemies or watchers (each in its own squad, without orders)
fn enemies(w: &WorldConfig, tiles: &[Tile], args: &Args) -> Vec<oc_individual::Individual> {
    match args.case {
        TestCase::TankMoveDuringTarget => vec![utils::soldier(
            w,
            Side::B,
            utils::on_ground(w, tiles, TMDT_ENEMY.into()),
        )],
        TestCase::TankChassisVisible => vec![utils::soldier(
            w,
            Side::A,
            utils::on_ground(w, tiles, TCV_WATCHER.into()),
        )],
        TestCase::TankLov => vec![
            utils::soldier(w, Side::A, utils::on_ground(w, tiles, TL_WATCHER.into())),
            utils::soldier(w, Side::B, utils::on_ground(w, tiles, TL_HIDDEN.into())),
            utils::soldier(w, Side::B, utils::on_ground(w, tiles, TL_CONTROL.into())),
        ],
        TestCase::TankMoveForward
        | TestCase::TankMoveRotation
        | TestCase::TankTarget
        | TestCase::TankFire
        | TestCase::TankProjectiles
        | TestCase::TankProjectileHitObject
        | TestCase::TankCrewStunnedByExplosion
        | TestCase::TankCrewKilledByExplosion
        | TestCase::TankCrewStunnedByTank
        | TestCase::TankCrewKilledByTank
        | TestCase::DrawTanks => vec![],
    }
}

#[cfg(feature = "test")]
fn obstacles(tiles: &[Tile], mod_: &Mod) -> Vec<oc_geo::tile::WorldTileIndex> {
    let nature = mod_.nature_index(BRICKWALL).unwrap();
    tiles
        .iter()
        .filter(|tile| tile.nature == nature)
        .map(|tile| tile.i)
        .collect()
}

fn squads(
    individuals: &[oc_individual::Individual],
    tanks: &[TankSetup],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    let crew_squads = |orders| crew_squads(individuals, tanks, orders);
    match args.case {
        TestCase::TankMoveForward => crew_squads(vec![Order::MoveTo(TMF_TARGET.into())]),
        TestCase::TankMoveRotation => crew_squads(vec![Order::MoveTo(TMR_TARGET.into())]),
        TestCase::TankMoveDuringTarget => crew_squads(vec![Order::MoveTo(TMDT_TARGET.into())]),
        TestCase::TankTarget => crew_squads(vec![Order::Suppress(TT_TARGET.into())]),
        TestCase::TankFire => crew_squads(vec![Order::Suppress(TF_TARGET.into())]),
        TestCase::TankProjectileHitObject => crew_squads(vec![Order::Suppress(TPHO_TARGET.into())]),
        TestCase::TankCrewStunnedByTank | TestCase::TankCrewKilledByTank => {
            crew_squads(vec![Order::Suppress(VH_TANK.into())])
        }
        TestCase::TankProjectiles
        | TestCase::TankChassisVisible
        | TestCase::TankLov
        | TestCase::TankCrewStunnedByExplosion
        | TestCase::TankCrewKilledByExplosion
        | TestCase::DrawTanks => crew_squads(vec![]),
    }
}

fn crew_squads(
    individuals: &[oc_individual::Individual],
    tanks: &[TankSetup],
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

    // Only (ordered) vehicle crews receive `orders`
    squads
        .into_iter()
        .map(|(vehicle, members)| {
            let leader = &individuals[members[0].0 as usize];
            let orders = match vehicle {
                Some(vehicle) if tanks[vehicle.0 as usize].ordered => orders.clone(),
                _ => vec![],
            };
            TestSquad::builder()
                .side(leader.side)
                .position(leader.position.into())
                .members(members)
                .orders(orders)
                .build()
                .make()
        })
        .collect()
}

fn vehicles(w: &WorldConfig, tiles: &[Tile], mod_: &Mod, tanks: &[TankSetup]) -> Vec<Vehicle> {
    let shell = mod_.shell_from_name(TF_SHELL).unwrap().index();

    tanks
        .iter()
        .map(|tank| {
            Vehicle::new(
                tank.side,
                mod_.vehicle_from_name(tank.type_).unwrap().index(),
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
        | TestCase::TankMoveDuringTarget
        | TestCase::TankTarget
        | TestCase::TankFire
        | TestCase::TankProjectileHitObject
        | TestCase::TankChassisVisible
        | TestCase::TankLov
        | TestCase::TankCrewStunnedByExplosion
        | TestCase::TankCrewKilledByExplosion
        | TestCase::TankCrewStunnedByTank
        | TestCase::TankCrewKilledByTank
        | TestCase::DrawTanks => vec![],
    }
}

/// `TF_SHELL` explosions sent to the server at game start (requires `debug` feature)
fn explosions(args: &Args) -> Vec<WorldVec2> {
    match args.case {
        TestCase::TankCrewStunnedByExplosion | TestCase::TankCrewKilledByExplosion => {
            vec![VH_TANK.into()]
        }
        TestCase::TankMoveForward
        | TestCase::TankMoveRotation
        | TestCase::TankMoveDuringTarget
        | TestCase::TankTarget
        | TestCase::TankFire
        | TestCase::TankProjectiles
        | TestCase::TankProjectileHitObject
        | TestCase::TankChassisVisible
        | TestCase::TankLov
        | TestCase::TankCrewStunnedByTank
        | TestCase::TankCrewKilledByTank
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
    use oc_individual::{IndividualIndex, squad::SquadIndex};
    use oc_root::geo::WorldVec2;
    use oc_root::{Wcfg, WorldConfig};
    use oc_utils::d2::Direction;
    use oc_utils::let_some;
    use oc_vehicle::VehicleIndex;

    use oc_geo::tile::WorldTileIndex;
    use oc_projectile::ProjectileId;
    use oc_world_server::{state::ObjectId, tracker::Tracker_};

    use crate::examples::vehicles::{TL_CONTROL_INDEX, TL_HIDDEN_INDEX, TMDT_ENEMY_INDEX, TMR_TARGET};

    use super::{
        Args, TF_TARGET, TF_TOLERANCE, TMF_TARGET, TPHO_TARGET, TT_TARGET, TT_TOLERANCE, TestCase,
        VH_CREW, VH_STUN_DURATION, VH_STUN_TOLERANCE,
    };

    /// Set when an explosion is spawned at `TF_TARGET`
    static TANK_FIRE_EXPLOSION: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);

    /// Set when an explosion is spawned (anywhere but at `TPHO_TARGET`)
    static TANK_FIRE_OBSTACLE_EXPLOSION: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);

    /// Record explosion spawned at the tank fire target (or elsewhere)
    pub fn on_fx(fx: On<oc_battle_gui::fx::FxEvent>) {
        use oc_physics::fx::{Fx, Visual};

        let Fx::Visual(Visual::Explosion(position, _)) = &fx.0 else {
            return;
        };
        let position: Vec2 = (*position).into();
        if position.distance(TF_TARGET.into()) <= TF_TOLERANCE {
            TANK_FIRE_EXPLOSION.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if position.distance(TPHO_TARGET.into()) > TF_TOLERANCE {
            TANK_FIRE_OBSTACLE_EXPLOSION.store(true, std::sync::atomic::Ordering::Relaxed);
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
        vehicles: Query<(&oc_battle_gui::ingame::vehicle::VehicleIndex, &Visibility)>,
        individuals: Query<(
            &oc_battle_gui::entity::individual::IndividualIndex,
            &Visibility,
        )>,
    ) {
        let_some!(w = &w.0, return);
        static SUCCESS_SINCE: Mutex<Option<Instant>> = Mutex::new(None);

        let speed = w.speed();
        let timeout = match args.0.case {
            TestCase::TankMoveForward | TestCase::TankMoveRotation => Duration::from_secs(60),
            // Move (80m at 5m/s), turret rotation, aim, reload and shell flights
            TestCase::TankMoveDuringTarget => Duration::from_secs(60),
            TestCase::TankTarget => Duration::from_secs(30),
            // Turret rotation (90° at 45°/s) then shell flight
            TestCase::TankFire => Duration::from_secs(30),
            // Aim, reload, then shell flight
            TestCase::TankProjectileHitObject => Duration::from_secs(30),
            // Let projectiles cross the tank (~3s at Weapon1 velocity)
            TestCase::TankProjectiles => Duration::from_secs(6),
            // Let visibilities be computed
            TestCase::TankChassisVisible | TestCase::TankLov => Duration::from_secs(10),
            TestCase::TankCrewKilledByExplosion => Duration::from_secs(10),
            // Aim, then shell flight
            TestCase::TankCrewKilledByTank => Duration::from_secs(30),
            // Stun duration
            TestCase::TankCrewStunnedByExplosion => Duration::from_secs(60),
            // Aim, shell flight, then stun duration
            TestCase::TankCrewStunnedByTank => Duration::from_secs(80),
            TestCase::DrawTanks => Duration::from_secs(5),
        };
        let timeout = timeout.div_f32(speed).max(Duration::from_secs(5));

        let success = match args.0.case {
            TestCase::TankMoveForward => move_accomplished(&world, TMF_TARGET),
            TestCase::TankMoveRotation => move_accomplished(&world, TMR_TARGET),
            TestCase::TankMoveDuringTarget => world
                .individual(IndividualIndex(TMDT_ENEMY_INDEX))
                .is_some_and(|enemy| enemy.status.is_dead()),
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
            // Shell exploded (out of the target): what it hit is checked after app exit, from
            // server tracker (see `check_tracker`)
            TestCase::TankProjectileHitObject => {
                TANK_FIRE_OBSTACLE_EXPLOSION.load(std::sync::atomic::Ordering::Relaxed)
            }
            // Enemy tank rendered visible (by its chassis, its position and crew being hidden)
            TestCase::TankChassisVisible => vehicles.iter().any(|(i, visibility)| {
                i.0 == VehicleIndex(0) && *visibility == Visibility::Visible
            }),
            // Lov path to the enemy behind the chassis is blocked (opacity), not the control one
            TestCase::TankLov => {
                let visible = |index: u64| {
                    individuals.iter().find_map(|(i, v)| {
                        (i.0 == IndividualIndex(index)).then_some(*v == Visibility::Visible)
                    })
                };
                if visible(TL_HIDDEN_INDEX) == Some(true) {
                    eprintln!("❌ FAILED ! Enemy behind the tank chassis is visible");
                    commands.write_message(bevy::app::AppExit::from_code(1));
                    return;
                }
                visible(TL_CONTROL_INDEX) == Some(true) && visible(TL_HIDDEN_INDEX) == Some(false)
            }
            // Crew killed and tank destroyed (penetrated)
            TestCase::TankCrewKilledByExplosion | TestCase::TankCrewKilledByTank => {
                VH_CREW.iter().all(|i| {
                    world
                        .individual(IndividualIndex(*i))
                        .is_some_and(|individual| individual.status.is_dead())
                }) && world
                    .vehicle(VehicleIndex(0))
                    .is_some_and(|vehicle| vehicle.destroyed)
            }
            TestCase::TankCrewStunnedByExplosion | TestCase::TankCrewStunnedByTank => {
                if world
                    .vehicle(VehicleIndex(0))
                    .is_some_and(|vehicle| vehicle.destroyed)
                {
                    eprintln!("❌ FAILED ! Tank is destroyed ({:?})", args.0.case);
                    commands.write_message(bevy::app::AppExit::from_code(1));
                    return;
                }
                match crew_stun_duration(&world, w, speed) {
                    CrewStun::Pending => false,
                    CrewStun::Recovered(duration) => {
                        let ok = (duration - VH_STUN_DURATION).abs() <= VH_STUN_TOLERANCE;
                        if !ok {
                            eprintln!(
                                "❌ FAILED ! Crew stunned during {duration}s, expected {VH_STUN_DURATION}s ({:?})",
                                args.0.case
                            );
                            commands.write_message(bevy::app::AppExit::from_code(1));
                            return;
                        }
                        true
                    }
                    CrewStun::Dead => {
                        eprintln!("❌ FAILED ! Crew is dead ({:?})", args.0.case);
                        commands.write_message(bevy::app::AppExit::from_code(1));
                        return;
                    }
                }
            }
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

    enum CrewStun {
        /// Crew not yet stunned, or still stunned
        Pending,
        /// Whole crew was stunned, then recovered after this duration (game seconds)
        Recovered(f32),
        /// A crew member is dead
        Dead,
    }

    /// Follow target tank crew status: record when the whole crew is stunned, then when it is
    /// operational again
    fn crew_stun_duration(world: &World, w: &WorldConfig, speed: f32) -> CrewStun {
        use oc_individual::Status;

        static STUNNED_SINCE: Mutex<Option<Instant>> = Mutex::new(None);
        static RECOVERED: Mutex<Option<f32>> = Mutex::new(None);

        let statuses: Vec<Status> = VH_CREW
            .iter()
            .filter_map(|i| world.individual(IndividualIndex(*i)))
            .map(|individual| individual.status)
            .collect();
        if statuses.len() != VH_CREW.len() {
            return CrewStun::Pending;
        }
        if statuses.iter().any(|status| status.is_dead()) {
            return CrewStun::Dead;
        }
        if let Some(duration) = *RECOVERED.lock().unwrap() {
            return CrewStun::Recovered(duration);
        }

        let mut since = STUNNED_SINCE.lock().unwrap();
        let stunned = statuses
            .iter()
            .all(|status| matches!(status, Status::Stunned(_)));
        let operational = statuses
            .iter()
            .all(|status| matches!(status, Status::Operational));
        match (*since, stunned, operational) {
            (None, true, _) => {
                // Stun may already be decreased when first seen (client update latency, mainly
                // at game start): take into account the already elapsed stun (one decrease per
                // game second)
                let stun = statuses
                    .iter()
                    .filter_map(|status| match status {
                        Status::Stunned(stun) => Some(stun.0),
                        _ => None,
                    })
                    .max()
                    .unwrap_or_default();
                let consumed = w.vehicle_crew_stunned_value().0.saturating_sub(stun) as f32
                    / w.individual_tick_decrease_stunned().0.max(1) as f32;
                let now = Instant::now();
                *since = Some(
                    now.checked_sub(Duration::from_secs_f32(consumed / speed))
                        .unwrap_or(now),
                );
            }
            (Some(since), _, true) => {
                let duration = since.elapsed().as_secs_f32() * speed;
                *RECOVERED.lock().unwrap() = Some(duration);
                return CrewStun::Recovered(duration);
            }
            _ => {}
        }

        CrewStun::Pending
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
    pub fn check_tracker(args: &Args, obstacles: &[WorldTileIndex], tracker: &Tracker_) {
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
            TestCase::TankProjectileHitObject => {
                // Tank shell is the only projectile: it must collide with an obstacle tile
                let hit = tracker.physics.iter().any(|event| match event {
                    oc_physics::Event::Collision(a, b, _) => matches!(
                        (a, b),
                        (ObjectId::Projectile(_), ObjectId::Tile(tile))
                        | (ObjectId::Tile(tile), ObjectId::Projectile(_))
                        if obstacles.contains(tile)
                    ),
                    _ => false,
                });

                if !hit {
                    eprintln!("❌ FAILED ! Shell did not hit obstacle ({:?})", args.case);
                    std::process::exit(1);
                }
                println!("✅ SUCCESS ! Shell hit obstacle ({:?})", args.case);
            }
            TestCase::TankMoveForward
            | TestCase::TankMoveRotation
            | TestCase::TankMoveDuringTarget
            | TestCase::TankTarget
            | TestCase::TankFire
            | TestCase::TankChassisVisible
            | TestCase::TankLov
            | TestCase::TankCrewStunnedByExplosion
            | TestCase::TankCrewKilledByExplosion
            | TestCase::TankCrewStunnedByTank
            | TestCase::TankCrewKilledByTank
            | TestCase::DrawTanks => {}
        }
    }
}
