#![cfg_attr(not(feature = "test"), allow(dead_code, unused_imports))]
#![allow(clippy::unwrap_used)]

use oc_utils::index::AtIndex;
#[cfg(feature = "test")]
use std::time::Instant;
use std::{path::PathBuf, time::Duration};

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use bevy::prelude::*;
use clap::{Parser, ValueEnum};
use oc_battle_gui::ingame::{GameConfigReceived, camera::move_::CenterCameraOn};
#[cfg(any(feature = "test", feature = "debug"))]
use oc_battle_gui::states::GameConfig;
use oc_geo::tile::WorldTileIndex;
use oc_individual::order::Order;
use oc_mod::{Mod, shell::ShellIndex};
use oc_root::{
    WcfgInto, WorldConfig, battle::BattlePhase, geo::WorldVec2, physics::Meters, side::Side,
    utils::Frequency,
};
#[cfg(any(feature = "test", feature = "debug"))]
use oc_utils::let_some;
#[cfg(feature = "test")]
use oc_world::interior::Interior;
use oc_world::{load::WorldPath, meta::Meta, tile::Tile};
use tests::{individual::TestIndividual, squad::TestSquad};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: Case,

    #[arg(long, action)]
    test: bool,

    /// Mod directory to use. Shell indexes of cases are not checked against it
    #[arg(long = "mod", default_value = "mods/tests1")]
    mod_: PathBuf,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Case {
    /// Individual (our side) enter an interior show this interior
    Enter,
    /// Individual (our side) exit an interior hide this interior
    Out,
    /// Individual inside an interior survives an explosion outside it: wall resistance is enough
    OutsideExplosionProtected,
    /// Individual inside an interior dies of an explosion outside it: wall resistance is not enough
    OutsideExplosionNotProtected,
    /// Individual inside a protected interior survives a non penetrating explosion inside it
    OnTheTopExplosionProtected,
    /// Individual inside an interior dies of a non penetrating explosion inside it, because a first
    /// explosion destructed the interior tiles
    OnTheTopExplosionNotProtectedBecauseBroken,
    /// Individual inside an interior without protection dies of an explosion inside it
    OnTheTopExplosionNotProtected,
    /// Squad of three individuals inside a bunker survives a salvo of explosions near it
    NearExplosionProtectedInBunker,
}

impl Case {
    fn is_explosion(&self) -> bool {
        matches!(
            self,
            Case::OutsideExplosionProtected
                | Case::OnTheTopExplosionProtected
                | Case::OnTheTopExplosionNotProtected
                | Case::OnTheTopExplosionNotProtectedBecauseBroken
                | Case::OutsideExplosionNotProtected
                | Case::NearExplosionProtectedInBunker
        )
    }

    fn individual_positions(&self) -> Vec<[f32; 2]> {
        match self {
            Case::OnTheTopExplosionNotProtected
            | Case::OnTheTopExplosionNotProtectedBecauseBroken
            | Case::OutsideExplosionNotProtected => vec![[111., 192.]],
            Case::NearExplosionProtectedInBunker => {
                vec![[167.5, 357.5], [158., 351.], [178., 364.]]
            }
            _ => vec![[185., 344.]],
        }
    }

    /// Explosions to fire: delay since the first one, shell and position
    fn explosions(&self, speed: f32) -> Vec<(Duration, ShellIndex, [f32; 2])> {
        let second = Duration::from_secs(3).div_f32(speed);
        match self {
            Case::OutsideExplosionProtected => {
                vec![(Duration::ZERO, EXPLOSION_PROTECTED_SHELL, [255., 356.])]
            }
            Case::OnTheTopExplosionProtected => {
                vec![(Duration::ZERO, EXPLOSION_PROTECTED_SHELL, [180., 341.])]
            }
            Case::OnTheTopExplosionNotProtected => {
                vec![(Duration::ZERO, EXPLOSION_PROTECTED_SHELL, [111., 192.])]
            }
            Case::OnTheTopExplosionNotProtectedBecauseBroken => vec![
                (Duration::ZERO, DESTRUCTION_SHELL, [111., 192.]),
                // On a wall tile of the interior, destructed by the first explosion
                (second, NO_PENETRATION_SHELL, [117.5, 152.5]),
            ],
            Case::OutsideExplosionNotProtected => {
                vec![(Duration::ZERO, EXPLOSION_NOT_PROTECTED_SHELL, [118., 211.])]
            }
            Case::NearExplosionProtectedInBunker => (0..10)
                .map(|i| {
                    (
                        Duration::from_millis(250 * i).div_f32(speed),
                        NAVAL_SHELL,
                        [167., 365.],
                    )
                })
                .collect(),
            Case::Enter | Case::Out => unreachable!(),
        }
    }

    /// Time after which the individual is expected to be (or not) dead
    fn test_duration(&self, speed: f32) -> Duration {
        let last = self
            .explosions(speed)
            .last()
            .map(|e| e.0)
            .unwrap_or_default();
        last + Duration::from_secs(5).div_f32(speed).max(MIN_TEST_DURATION)
    }

    fn expect_dead(&self) -> bool {
        matches!(
            self,
            Case::OnTheTopExplosionNotProtected
                | Case::OnTheTopExplosionNotProtectedBecauseBroken
                | Case::OutsideExplosionNotProtected
        )
    }
}

/// "5/38 shell"
const NAVAL_SHELL: ShellIndex = ShellIndex(0);
/// "Kill & Stun only (no shrapnel & 0.1 destruction force)": walls resist
const EXPLOSION_PROTECTED_SHELL: ShellIndex = ShellIndex(2);
/// "Kill & Stun only (no shrapnel & 1.0 destruction force)": walls do not resist
const EXPLOSION_NOT_PROTECTED_SHELL: ShellIndex = ShellIndex(4);
/// "Kill & Stun only (no shrapnel & 0.0 penetration)": interior is not penetrated, unless broken
const NO_PENETRATION_SHELL: ShellIndex = ShellIndex(5);
/// "Destruction Only (10 meters)"
const DESTRUCTION_SHELL: ShellIndex = ShellIndex(3);

pub fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse_from(crate::args());
    if args.test {
        #[cfg(any(not(feature = "test"), not(feature = "debug")))]
        {
            panic!("To enable test, feature `test` and `debug` must be enabled too")
        }
    }

    logging::setup_logging()?;

    let mod__ = args.mod_.clone();
    let mod_ = Mod::load(&mod__, None)?;
    let map_ = PathBuf::from("maps/minidblue");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let world = Meta::from_file(&map_.meta());
    let world = world.context(format!("Read file {}", map_.meta().display()))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(world.geo_meters_per_z));
    let speed = w.speed();
    let w = w.with_interiors_tick(Frequency::each(Duration::from_secs(1).div_f32(speed)));
    let tiles = map.tiles(&w, &mod_).unwrap();

    let individuals = individuals(&w, &mod_, &tiles, &args);
    let squads = squads(&w, &tiles, &individuals, &args);
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
        .mod_(mod__)
        .snapshot(snapshot)
        .test_app_exit_code(args.test);

    let example = example.install(Box::new(install));

    example.build().run()?;

    Ok(())
}

fn individuals(
    w: &WorldConfig,
    _mod_: &oc_mod::Mod,
    tiles: &[Tile],
    args: &Args,
) -> Vec<oc_individual::Individual> {
    match args.case {
        Case::Enter => {
            let position = WorldVec2::new(915., 50.);
            let tile: WorldTileIndex = position.into_(w);
            let tile = tiles.at(tile);
            let position = position.extend(tile.z_pixels(w));
            vec![
                TestIndividual::builder()
                    .side(Side::A)
                    .position(position)
                    .build()
                    .make(w),
            ]
        }
        Case::Out => {
            let position = WorldVec2::new(850., 70.);
            let tile: WorldTileIndex = position.into_(w);
            let tile = tiles.at(tile);
            let position = position.extend(tile.z_pixels(w));
            vec![
                TestIndividual::builder()
                    .side(Side::A)
                    .position(position)
                    .build()
                    .make(w),
            ]
        }
        Case::OutsideExplosionProtected
        | Case::OnTheTopExplosionProtected
        | Case::OnTheTopExplosionNotProtected
        | Case::OnTheTopExplosionNotProtectedBecauseBroken
        | Case::OutsideExplosionNotProtected
        | Case::NearExplosionProtectedInBunker => args
            .case
            .individual_positions()
            .into_iter()
            .map(|position| {
                let position = WorldVec2::from(position);
                let tile: WorldTileIndex = position.into_(w);
                let tile = tiles.at(tile);
                let position = position.extend(tile.z_pixels(w));
                TestIndividual::builder()
                    .side(Side::A)
                    .position(position)
                    .build()
                    .make(w)
            })
            .collect(),
    }
}

fn squads(
    _w: &WorldConfig,
    _tiles: &[Tile],
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        Case::Enter => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::MoveFastTo(WorldVec2::new(850., 70.))])
                .build()
                .make(),
        ],
        Case::Out => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::MoveFastTo(WorldVec2::new(915., 50.))])
                .build()
                .make(),
        ],
        Case::OutsideExplosionProtected
        | Case::OnTheTopExplosionProtected
        | Case::OnTheTopExplosionNotProtected
        | Case::OnTheTopExplosionNotProtectedBecauseBroken
        | Case::OutsideExplosionNotProtected
        | Case::NearExplosionProtectedInBunker => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(
                    (0..individuals.len() as u64)
                        .map(oc_individual::IndividualIndex)
                        .collect(),
                )
                .build()
                .make(),
        ],
    }
}

fn install(app: &mut bevy::app::App) {
    let args = Args::parse_from(crate::args());

    match args.case {
        Case::Enter
        | Case::Out
        | Case::OutsideExplosionProtected
        | Case::OnTheTopExplosionProtected
        | Case::OnTheTopExplosionNotProtected
        | Case::OnTheTopExplosionNotProtectedBecauseBroken
        | Case::OutsideExplosionNotProtected
        | Case::NearExplosionProtectedInBunker => {
            // Center screen on the house
            app.add_observer(|_: On<GameConfigReceived>, mut commands: Commands| {
                commands.trigger(CenterCameraOn(WorldVec2::new(885., 50.)));
            });
        }
    }

    if args.case.is_explosion() {
        app.add_observer(|_: On<GameConfigReceived>, mut commands: Commands| {
            commands.trigger(CenterCameraOn(WorldVec2::new(220., 350.)));
        });

        #[cfg(feature = "debug")]
        app.init_resource::<Fired>()
            .add_systems(Update, fire_explosion);

        #[cfg(feature = "test")]
        if args.test {
            app.init_resource::<ExplosionProtectedTest>()
                .add_systems(Update, explosion_protected_test);
        }
    } else {
        #[cfg(feature = "test")]
        if args.test {
            let state = State {
                start: Instant::now(),
            };
            app.insert_resource(state)
                .add_systems(Update, test_interior_visibility);
        }
    }
}

#[cfg(feature = "debug")]
fn fire_explosion(
    mut commands: Commands,
    mut start: Local<Option<std::time::Instant>>,
    mut fired: ResMut<Fired>,
    individuals: Query<&oc_battle_gui::entity::individual::IndividualIndex>,
    g: Res<GameConfig>,
) {
    use oc_battle_gui::network::output::ToServerEvent;
    use oc_network::ToServer;

    // Do not fire if individual is not yet displayed
    if !individuals
        .iter()
        .any(|i| i.0 == oc_individual::IndividualIndex(0))
    {
        return;
    }

    let_some!(g = &g.0, return);
    let start = *start.get_or_insert_with(std::time::Instant::now);
    let explosions = Args::parse_from(crate::args()).case.explosions(g.w.speed());
    fired.frames_since_last = fired.frames_since_last.saturating_add(1);
    while let Some((delay, shell, position)) = explosions.get(fired.count) {
        if start.elapsed() < *delay {
            break;
        }
        commands.trigger(ToServerEvent(ToServer::DebugExplosion(
            *shell,
            WorldVec2::from(*position),
        )));
        fired.count += 1;
        fired.frames_since_last = 0;
    }
}

const MIN_TEST_DURATION: Duration = Duration::from_millis(500);
/// Frames to wait after last explosion before checking result: explosion effects are received by
/// network, so test must not depend only on (wall clock) duration when frames are slow
#[cfg(feature = "test")]
const MIN_FRAMES_AFTER_LAST_EXPLOSION: u32 = 10;

/// Count of explosions fired, and frames elapsed since the last one
#[cfg(feature = "debug")]
#[derive(Debug, Resource, Default)]
struct Fired {
    count: usize,
    frames_since_last: u32,
}

#[cfg(feature = "test")]
#[derive(Debug, Resource, Default)]
struct ExplosionProtectedTest {
    fired: Option<Instant>,
}

/// Individual must be dead (or not, according to the case) after explosion.
#[cfg(feature = "test")]
fn explosion_protected_test(
    mut commands: Commands,
    mut test: ResMut<ExplosionProtectedTest>,
    g: Res<GameConfig>,
    fired_: Res<Fired>,
    individuals: Query<(
        &oc_battle_gui::entity::individual::IndividualIndex,
        &oc_battle_gui::ingame::individual::Status,
    )>,
) {
    let_some!(g = &g.0, return);
    let expected = Args::parse_from(crate::args())
        .case
        .individual_positions()
        .len();
    if individuals.iter().count() < expected {
        return;
    }
    let fired = *test.fired.get_or_insert_with(Instant::now);

    let dead = individuals
        .iter()
        .any(|(_, status)| matches!(status.0, oc_individual::Status::Dead));
    let case = Args::parse_from(crate::args()).case;
    let all_fired = fired_.count == case.explosions(g.w.speed()).len()
        && fired_.frames_since_last >= MIN_FRAMES_AFTER_LAST_EXPLOSION;
    let elapsed = all_fired && fired.elapsed() > case.test_duration(g.w.speed());
    if !(dead || elapsed) {
        return;
    }

    if dead == Args::parse_from(crate::args()).case.expect_dead() {
        println!("✅ Success !");
        commands.write_message(bevy::app::AppExit::from_code(0));
    } else {
        eprintln!("❌ Test failed ! Individual dead: {dead}");
        commands.write_message(bevy::app::AppExit::from_code(1));
    }
}

#[cfg(feature = "test")]
#[derive(Debug, Resource)]
struct State {
    start: Instant,
}

#[cfg(feature = "test")]
fn test_interior_visibility(
    mut commands: Commands,
    g: Res<GameConfig>,
    query: Query<(&Interior, &Visibility)>,
    state: Res<State>,
) {
    let_some!(g = &g.0, return);
    let speed = g.w.speed();
    let args = Args::parse_from(crate::args());

    let timeout = match args.case {
        Case::Enter | Case::Out => Duration::from_secs(10),
        _ => return,
    }
    .div_f32(speed)
    // Success is detected as soon as it happens: a large minimal timeout does not slow down the
    // test, and let time to (maybe slow) app start and rendering
    .max(Duration::from_secs(15));

    if match args.case {
        Case::Enter => query.iter().any(|(_, v)| matches!(v, Visibility::Visible)),
        Case::Out => {
            state.start.elapsed().as_secs_f32() > 2. / speed
                && query.iter().any(|(_, v)| matches!(v, Visibility::Hidden))
        }
        _ => false,
    } {
        println!("✅ Success !");
        commands.write_message(bevy::app::AppExit::from_code(0));
    }

    if state.start.elapsed() > timeout {
        println!("❌ Test failed !");
        commands.write_message(bevy::app::AppExit::from_code(1));
    }
}
