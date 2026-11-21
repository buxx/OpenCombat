#![allow(clippy::unwrap_used)]

use std::{path::PathBuf, time::Duration};

use crate::{
    logging, run,
    snapshot::SnapshotBuilder,
    utils::{self, Firing, Loaded, armed_soldier, direction, soldier},
};
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
use oc_root::{WorldConfig, side::Side};
use oc_utils::let_some;
use tests::squad::TestSquad;

const MOD: &str = "mods/tests1";

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
    utils::ensure_test_feature(args.test);

    let mod_ = PathBuf::from(MOD);
    let map = PathBuf::from("maps/height");
    let Loaded {
        mod_: mod__,
        map: map_,
        w,
        ..
    } = Loaded::new(&mod_, &map)?;

    #[cfg(feature = "debug")]
    if let TestCase::IndividualShotTraverseVolume | TestCase::IndividualShotBlockedByVolume =
        args.case
    {
        WorldConfig::set_inaccuracy_spread_enabled(true);
        WorldConfig::set_inaccuracy_spread(0.0);
    }

    let (individuals, squads) = individuals_and_squads(&args, &w, &mod__);
    let snapshot = SnapshotBuilder::fight(map_, individuals, squads).build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
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
            use utils::tracked;

            let tracker = tracker.take();
            let collisions = tracked::tile_collisions(&tracker);

            match args.case {
                // We consider success if physics event own at least 10 projectiles collisions
                TestCase::MultipleAgainstWall | TestCase::MultipleAgainstHill => {
                    assert!(collisions >= 10)
                }
                TestCase::OneAgainstWall | TestCase::OneAgainstHill => assert!(collisions == 1),
                // Soldier B (index 1) must be killed
                TestCase::IndividualShotTraverseVolume => {
                    assert!(tracked::dead(&tracker, IndividualIndex(1)))
                }
                TestCase::IndividualShotBlockedByVolume => {
                    // Soldier A (index 0) must still be operational
                    assert!(!tracked::dead(&tracker, IndividualIndex(0)));
                    // Ensure soldier B really shot (else test is meaningless)
                    assert!(collisions > 0);
                }
                TestCase::OneTraverseVolumes | TestCase::OneStartInsideWall => {
                    assert!(collisions == 0)
                }
            };
            println!("✅ Test success");
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
                armed_soldier(w, mod_, Side::A, [123., 127., 0.], "Weapon3"),
                soldier(w, Side::B, [180., 125., 0.]),
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
                soldier(w, Side::A, [123., 127., 0.]),
                armed_soldier(w, mod_, Side::B, [245., 125., 19.5], "Weapon3"),
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
    let mod_ = Mod::load(&PathBuf::from(MOD), None).unwrap();

    // Weapon2 and Weapon3 fire Weapon1 ammunition (and shot, for Weapon2)
    let weapon = |name| mod_.weapons.iter().find(|w| w.name() == name).unwrap();
    let firing1 = Firing::weapon1(&mod_);
    let firing2 = Firing {
        weapon: weapon("Weapon2").index(),
        ..firing1
    };
    let weapon3 = weapon("Weapon3");
    let firing3 = Firing {
        weapon: weapon3.index(),
        shot: weapon3
            .shots()
            .iter()
            .find(|s| s.name() == "Burst3")
            .unwrap()
            .index(),
        ..firing1
    };

    // Projectiles: firing, repeat, from and to (ten directions)
    let projectiles = match args.case {
        TestCase::MultipleAgainstHill => vec![
            (firing1, 10, [70., 60., 8.5], [100., 60., 8.5]),
            (firing2, 10, [70., 65., 8.5], [100., 65., 8.5]),
            (firing3, 10, [70., 70., 8.5], [100., 70., 8.5]),
        ],
        TestCase::OneAgainstHill => vec![(firing1, 1, [70., 60., 8.5], [100., 60., 8.5])],
        TestCase::MultipleAgainstWall => vec![
            (firing1, 10, [5., 125., 8.5], [100., 125., 8.5]),
            (firing2, 10, [5., 130., 8.5], [100., 130., 8.5]),
            (firing3, 10, [5., 135., 8.5], [100., 135., 8.5]),
        ],
        TestCase::OneAgainstWall => vec![(firing1, 1, [5., 125., 8.5], [100., 125., 8.5])],
        TestCase::OneTraverseVolumes => vec![(firing1, 1, [120., 145., 1.0], [120., 245., 1.0])],
        TestCase::OneStartInsideWall => vec![(firing1, 1, [122., 136., 8.5], [122., 236., 8.5])],
        TestCase::IndividualShotTraverseVolume | TestCase::IndividualShotBlockedByVolume => {
            vec![]
        }
    };

    for (firing, repeat, from, to) in projectiles {
        let spawn = firing.spawn(repeat, from, vec![direction(from, to); 10], Side::A);
        commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
    }
}
