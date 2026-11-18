#![allow(clippy::unwrap_used)]

use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use bevy::prelude::*;

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use clap::{Parser, ValueEnum};
use oc_battle_gui::{
    ingame::{
        FirstIngameEnter,
        camera::GoToPoint,
        individual::ForgotIndividual,
        input::{individual::InsertIndividualEvent, projectile::InsertProjectileEvent},
        projectile::ForgotProjectile,
    },
    network::output::ToServerEvent,
    states::Game,
};
use oc_individual::{IndividualIndex, order::Order};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_projectile::{ProjectileId, spawn::SpawnProjectiles};
use oc_root::{
    Wcfg, WorldConfig,
    battle::BattlePhase,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    side::Side,
};
use oc_world::{meta::Meta, tile::Tile};
use tests::{individual::TestIndividual, squad::TestSquad};

const AFTER_SUCCESS_WAIT: Duration = Duration::from_secs(1);
const MOD: &str = "mods/tests1";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,

    #[arg(long, action)]
    test: bool,
}
impl Args {
    fn timeout(&self) -> Duration {
        match self.case {
            TestCase::ProjectileMoveIn => Duration::from_secs(5),
            TestCase::ProjectileMoveOut => Duration::from_secs(5),
            TestCase::IndividualMoveIn => Duration::from_secs(10),
            TestCase::IndividualMoveOut => Duration::from_secs(10),
            TestCase::IndividualAtCross => Duration::from_secs(5),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    ProjectileMoveIn,
    ProjectileMoveOut,
    IndividualMoveIn,
    IndividualMoveOut,
    IndividualAtCross,
}

pub fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());
    let mod_ = PathBuf::from(MOD);
    let mod__ = oc_mod::Mod::load(&mod_, None)?;
    let map = PathBuf::from("maps/meadow1");
    let meta = Meta::from_file(&map.join("meta.toml"))?;
    let map_ = oc_world::reader::MapReader::new(&map);
    let map_ = map_.context(format!("Read map_ {}", map.display()))?;
    let w = WorldConfig::new(map_.width().unwrap() as u64, map_.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    let tiles = map_.tiles(&w, &mod__).unwrap();

    let individuals = individuals(&w, &tiles, &args);
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
        .regions_width(2)
        .regions_height(2)
        .region_width(10)
        .region_height(10)
        .install(Box::new(install))
        .snapshot(snapshot)
        .test_app_exit_code(args.test);

    example.build().run()?;

    if args.test {
        if !SUCCESS.load(Ordering::Relaxed) {
            anyhow::bail!("❌ Test failed")
        }
        println!("✅ Test success !");
    }

    Ok(())
}

// FIXME BS NOW: in IndividualMoveOut case, two individuals are displayed. Its abnormal
fn individuals(w: &WorldConfig, _tiles: &Vec<Tile>, args: &Args) -> Vec<oc_individual::Individual> {
    match args.case {
        TestCase::ProjectileMoveIn => vec![],
        TestCase::ProjectileMoveOut => vec![],
        TestCase::IndividualMoveIn => vec![
            TestIndividual::builder()
                .position(WorldVec3::new(375., 248., 0.))
                .build()
                .make(w),
        ],
        TestCase::IndividualMoveOut => vec![
            TestIndividual::builder()
                .position(WorldVec3::new(210., 248., 0.))
                .build()
                .make(w),
        ],
        TestCase::IndividualAtCross => vec![
            TestIndividual::builder()
                .position(WorldVec3::new(250., 250., 0.))
                .build()
                .make(w),
        ],
    }
}

fn squads(
    _w: &WorldConfig,
    _tiles: &Vec<Tile>,
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        TestCase::ProjectileMoveIn => {
            vec![]
        }
        TestCase::ProjectileMoveOut => vec![],
        TestCase::IndividualMoveIn => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::MoveTo(WorldVec2::new(250., 250.))])
                .build()
                .make(),
        ],
        TestCase::IndividualMoveOut => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::MoveTo(WorldVec2::new(10., 250.))])
                .build()
                .make(),
        ],
        TestCase::IndividualAtCross => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![])
                .build()
                .make(),
        ],
    }
}

static SUCCESS: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Resource, Default)]
struct State {
    success: Option<Instant>,
    individuals: Vec<IndividualIndex>,
    projectiles: Vec<ProjectileId>,
}

fn install(app: &mut bevy::app::App) {
    let args = Args::parse_from(crate::args());

    if args.test {
        app.add_systems(Update, test_tracker);
    }

    app.init_resource::<State>()
        .add_observer(on_first_ingame_enter);

    match args.case {
        TestCase::ProjectileMoveIn => {
            app.add_observer(on_insert_projectile);
        }
        TestCase::ProjectileMoveOut => {
            app.add_observer(on_remove_projectile);
        }
        TestCase::IndividualMoveIn => {
            app.add_observer(on_insert_individual);
        }
        TestCase::IndividualMoveOut => {
            app.add_observer(on_remove_individual);
        }
        TestCase::IndividualAtCross => {}
    }
}

fn test_tracker(mut commands: Commands, game: Res<Game>, state: ResMut<State>, w: Res<Wcfg>) {
    let args = Args::parse_from(crate::args());
    let speed = w.0.as_ref().map(|w| w.speed()).unwrap_or(1.0);
    let timeout = args.timeout().div_f32(speed);
    if game.started.elapsed() > timeout
        || state
            .success
            .map(|success| success.elapsed() > AFTER_SUCCESS_WAIT)
            .unwrap_or_default()
    {
        commands.write_message(bevy::app::AppExit::from_code(0));
    }
}

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    let args = Args::parse_from(crate::args());
    let mod_ = Mod::load(&PathBuf::from(MOD), None).unwrap();

    let weapon1 = mod_.weapons.iter().find(|w| w.name() == "Weapon1").unwrap();
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

    match args.case {
        TestCase::ProjectileMoveIn => {
            let directions = vec![
                (WorldVec3::new(100., 248., 8.5) - WorldVec3::new(450., 248., 8.5))
                    .normalize_or_zero(),
            ];
            let spawn = SpawnProjectiles::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                [450., 248., 8.5].into(),
                directions,
                Side::A,
                Duration::ZERO,
                None,
            );
            commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
        }
        TestCase::ProjectileMoveOut => {
            let directions = vec![
                (WorldVec3::new(100., 248., 8.5) - WorldVec3::new(250., 248., 8.5))
                    .normalize_or_zero(),
            ];
            let spawn = SpawnProjectiles::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                [250., 248., 8.5].into(),
                directions,
                Side::A,
                Duration::ZERO,
                None,
            );
            commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
        }
        TestCase::IndividualMoveIn => {}
        TestCase::IndividualMoveOut => {}
        TestCase::IndividualAtCross => {}
    }

    commands.trigger(GoToPoint(WorldVec2::new(250., 250.)));
}

fn on_insert_projectile(event: On<InsertProjectileEvent>, mut state: ResMut<State>) {
    state.projectiles.push(event.0);
    SUCCESS.store(true, Ordering::Relaxed);
}

fn on_remove_projectile(event: On<ForgotProjectile>, mut state: ResMut<State>) {
    state.projectiles.push(event.0);
    SUCCESS.store(true, Ordering::Relaxed);
}

fn on_insert_individual(event: On<InsertIndividualEvent>, mut state: ResMut<State>) {
    state.individuals.push(event.0);
    SUCCESS.store(true, Ordering::Relaxed);
}

fn on_remove_individual(event: On<ForgotIndividual>, mut state: ResMut<State>) {
    state.individuals.push(event.0);
    SUCCESS.store(true, Ordering::Relaxed);
}
