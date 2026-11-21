#![allow(clippy::unwrap_used)]

use std::{
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::{
    logging, run,
    snapshot::SnapshotBuilder,
    utils::{self, Firing, Loaded, direction},
};
use bevy::prelude::*;
use clap::{Parser, ValueEnum};
use oc_battle_gui::{
    ingame::{FirstIngameEnter, individual::Status},
    network::output::ToServerEvent,
    states::Game,
};
use oc_individual::{
    INDIVIDUAL_PRONE_VOLUME_WIDTH, IndividualIndex, order::Order, squad::SquadFormation,
};
use oc_mod::{Mod, illustration::IllustrationIndex};
use oc_network::ToServer;
use oc_root::{Wcfg, WorldConfig, side::Side};
use oc_utils::{d2::Direction, let_some};

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
    SamePixel,
    InVolume,
    DifferentTile,
    Above,
    AboveProne,
    NearRotatedProne,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());
    utils::ensure_test_feature(args.test);

    let mod_ = PathBuf::from("mods/tests1");
    let map = PathBuf::from("maps/meadow1");
    let Loaded {
        mod_: mod__,
        map: map_,
        w,
        ..
    } = Loaded::new(&mod_, &map)?;
    // Individuals are built with the default geo pixels per meters
    let (individuals, squads) = individuals(&args, &w);
    let w = w.with_geo_pixels_per_meters(5.0);
    let snapshot = SnapshotBuilder::fight(map_, individuals, squads).build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .install(Box::new(install))
        .test_app_exit_code(args.test)
        .snapshot(snapshot);

    #[cfg(feature = "test")]
    let tracker = example.build().run()?;
    #[cfg(not(feature = "test"))]
    example.build().run()?;

    if args.test {
        #[cfg(feature = "test")]
        {
            use utils::tracked;

            let tracker = tracker.take();
            let collision = tracked::individual_collisions(&tracker) > 0;
            let dead = tracked::dead(&tracker, IndividualIndex(0));

            match args.case {
                TestCase::SamePixel | TestCase::InVolume | TestCase::DifferentTile => {
                    assert!(collision);
                    assert!(dead);
                }
                TestCase::Above | TestCase::AboveProne | TestCase::NearRotatedProne => {
                    assert!(!collision);
                    assert!(!dead);
                }
            }

            println!("✅ (SERVER) All tests passed");
        }
    }

    Ok(())
}

fn individuals(
    args: &Args,
    w: &WorldConfig,
) -> (
    Vec<oc_individual::Individual>,
    Vec<oc_individual::squad::Squad>,
) {
    let positions = match args.case {
        TestCase::SamePixel => vec![[151.0, 151.0, 0.0]],
        TestCase::InVolume => vec![[150.0, 150.0, 0.0]],
        TestCase::DifferentTile => vec![[149.0, 149.0, 0.0]],
        TestCase::Above => vec![[151.0, 151.0, 0.0]],
        TestCase::AboveProne => vec![[151.0, 151.0, 0.0]],
        TestCase::NearRotatedProne => vec![[151.0, 151.0, 0.0]],
    };

    let individuals = positions
        .iter()
        .map(|p| {
            let position = (*p).into();
            oc_individual::Individual::fresh(w, Side::A, position)
        })
        .collect();

    let order = match args.case {
        TestCase::SamePixel | TestCase::InVolume | TestCase::DifferentTile | TestCase::Above => {
            Order::Idle
        }
        TestCase::AboveProne => Order::Hide(Direction::NORTH),
        TestCase::NearRotatedProne => Order::Hide(Direction::EST),
    };
    let squads = positions
        .iter()
        .enumerate()
        .map(|(i, position)| {
            let individual = IndividualIndex::from(i);
            oc_individual::squad::Squad {
                side: Side::A,
                illustration: IllustrationIndex(0),
                position: [position[0], position[1]].into(),
                leader: individual,
                members: vec![individual],
                actives: 2,
                formation: SquadFormation::Line,
                orders: vec![order.clone()],
                status: oc_individual::squad::SquadStatus::Operational,
            }
        })
        .collect();

    (individuals, squads)
}

fn install(app: &mut bevy::app::App) {
    let args = Args::parse_from(crate::args());

    if args.test {
        let (expected_status, expected_duration, timeout) = match args.case {
            TestCase::SamePixel | TestCase::InVolume | TestCase::DifferentTile => (
                oc_individual::Status::Dead,
                Duration::from_secs(1),
                Duration::from_secs(15),
            ),
            TestCase::Above | TestCase::AboveProne | TestCase::NearRotatedProne => (
                oc_individual::Status::Operational,
                Duration::from_secs(5),
                Duration::from_secs(15),
            ),
        };

        app.add_systems(
            Update,
            move |mut commands: Commands,
                  game: Res<Game>,
                  w: Res<Wcfg>,
                  individuals: Query<
                &Status,
                With<oc_battle_gui::entity::individual::IndividualIndex>,
            >| {
                let_some!(w = &w.0, return);
                let speed = w.speed();

                // Ensure a minimal timeout to let time to bevy to make "network" exchange with server
                let timeout = timeout.div_f32(speed).max(Duration::from_secs(5));

                // Store instant where individual is in expected status
                static STATUS_AS_EXPECTED_SINCE: Mutex<Option<Instant>> = Mutex::new(None);

                let mut status_as_expected_since = STATUS_AS_EXPECTED_SINCE.lock().unwrap();

                let status_as_expected = individuals
                    .iter()
                    .find(|status| status.0 == expected_status)
                    .is_some();

                match *status_as_expected_since {
                    Some(_) => {
                        // Status is still not or not anymore in expected status, keep or reset instant to None
                        if !status_as_expected {
                            *status_as_expected_since = None;
                        }
                    }
                    None => {
                        // Status just switched to expected, store the instant
                        if status_as_expected {
                            *status_as_expected_since = Some(Instant::now());
                        }
                    }
                }

                if (*status_as_expected_since)
                    .map(|s| s.elapsed() >= expected_duration.div_f32(speed))
                    .unwrap_or_default()
                {
                    println!("✅ (GUI) Individual is in expected status");
                    commands.write_message(bevy::app::AppExit::from_code(0));
                }

                if game.started.elapsed() > timeout {
                    eprintln!("❌ (GUI) Timeout reached ! Individual is not in expected status");
                    commands.write_message(bevy::app::AppExit::from_code(1));
                }
            },
        );
    }

    app.add_observer(on_first_ingame_enter);
}

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    let args = Args::parse_from(crate::args());
    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None).unwrap();
    let firing = Firing::weapon1(&mod_);

    let body_height = INDIVIDUAL_PRONE_VOLUME_WIDTH.0 * 5.0;
    let projectiles = match args.case {
        TestCase::SamePixel
        | TestCase::InVolume
        | TestCase::DifferentTile
        | TestCase::AboveProne => {
            vec![([220.0, 151.0, 5.0], [100.0, 151.0, 5.0])]
        }
        TestCase::Above => vec![([220.0, 151.0, 15.0], [100.0, 151.0, 15.0])],
        TestCase::NearRotatedProne => vec![(
            [220.0, 151.0 + body_height + 1.0, 1.0],
            [100.0, 151.0 + body_height + 1.0, 1.0],
        )],
    };

    for (start, end) in projectiles {
        let spawn = firing.spawn(1, start, vec![direction(start, end)], Side::B);
        commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
    }
}
