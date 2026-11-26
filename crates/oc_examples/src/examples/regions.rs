#![allow(clippy::unwrap_used)]

use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use bevy::prelude::*;

use crate::{
    logging, run,
    snapshot::SnapshotBuilder,
    utils::{self, Firing, Loaded, direction},
};
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
use oc_projectile::ProjectileId;
use oc_root::{Wcfg, WorldConfig, geo::WorldVec2, side::Side};

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
    let map = PathBuf::from("maps/meadow1");
    let Loaded {
        mod_: mod__,
        map: map_,
        w,
        ..
    } = Loaded::new(&mod_, &map)?;

    let individuals = individuals(&w, &args);
    let squads = squads(&individuals, &args);
    let snapshot = SnapshotBuilder::fight(map_, individuals, squads).build(w, &mod__)?;

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
        utils::test_result(&SUCCESS)?;
    }

    Ok(())
}

fn individuals(w: &WorldConfig, args: &Args) -> Vec<oc_individual::Individual> {
    let position = match args.case {
        TestCase::ProjectileMoveIn | TestCase::ProjectileMoveOut => return vec![],
        TestCase::IndividualMoveIn => [375., 248., 0.],
        TestCase::IndividualMoveOut => [210., 248., 0.],
        TestCase::IndividualAtCross => [250., 250., 0.],
    };
    vec![utils::soldier(w, Side::A, position)]
}

fn squads(
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    let orders = match args.case {
        TestCase::ProjectileMoveIn | TestCase::ProjectileMoveOut => return vec![],
        TestCase::IndividualMoveIn => vec![Order::MoveTo(WorldVec2::new(250., 250.))],
        TestCase::IndividualMoveOut => vec![Order::MoveTo(WorldVec2::new(10., 250.))],
        TestCase::IndividualAtCross => vec![],
    };
    vec![utils::squad(individuals, &[0], orders)]
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
    let firing = Firing::weapon1(&mod_);

    let from = match args.case {
        TestCase::ProjectileMoveIn => Some([450., 248., 8.5]),
        TestCase::ProjectileMoveOut => Some([250., 248., 8.5]),
        TestCase::IndividualMoveIn | TestCase::IndividualMoveOut | TestCase::IndividualAtCross => {
            None
        }
    };
    if let Some(from) = from {
        let directions = vec![direction(from, [100., 248., 8.5])];
        let spawn = firing.spawn(1, from, directions, Side::A);
        commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
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
