#![allow(clippy::unwrap_used)]

use crate::{logging, tests::behavior};
use bevy::prelude::*;
use clap::{Parser, ValueEnum};
#[cfg(feature = "debug")]
use oc_battle_gui::ingame::camera::squad::ToggleShowFormationPositions;
#[cfg(feature = "test")]
use oc_battle_gui::ingame::individual::{AccomplishedEvent, MoveStepAccomplishedEvent};
use oc_individual::order::Order;
#[cfg(feature = "debug")]
use oc_mod::shell::ShellIndex;
#[cfg(feature = "debug")]
use oc_utils::let_some;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,

    /// Number of squad members
    #[arg(long)]
    individual_count: usize,

    /// Number of frag grenades carried by each individual. Throw grenade cases give one grenade
    /// order by grenade, so it is also the number of explosions expected by individual.
    #[arg(long, default_value_t = 1)]
    grenade_count: usize,

    #[arg(long, action)]
    test: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    Idle,
    MoveStraightAhead,
    MoveStraightAheadObstacle,
    MoveFastStraightAhead,
    MoveFastStraightAheadObstacle,
    Stunned,
    /// Throw grenade on a target in range
    ThrowGrenade,
    /// Throw grenade on a target out of range: individual must move before
    ThrowGrenadeAfterMove,
    /// Idle individual throws its grenade when an enemy comes near
    ThrowGrenadeWhenEnnemyNear,
}

impl TestCase {
    /// Grenade target of throw cases
    fn throw_target(self) -> Option<[f32; 2]> {
        match self {
            TestCase::ThrowGrenade => Some(TG_TARGET),
            TestCase::ThrowGrenadeAfterMove => Some(TGAM_TARGET),
            _ => None,
        }
    }
}

const IDLE_LEADER_POS: [f32; 2] = [150., 150.];

const STUNNED_POS: [f32; 2] = [150., 150.];
#[cfg(feature = "debug")]
const STUNNED_MOVE_TARGET: [f32; 2] = [180., 150.];
#[cfg(feature = "debug")]
const STUNNED_EXPLOSION_DISTANCE: f32 = 5.;
#[cfg(feature = "debug")]
const STUNNED_SHELL_INDEX: ShellIndex = ShellIndex(1);

const TG_LEADER_POS: [f32; 2] = [150., 150.];
// 20 meters (< THROW_MAX_DISTANCE)
const TG_TARGET: [f32; 2] = [250., 150.];
// 50 meters (> THROW_MAX_DISTANCE)
const TGAM_TARGET: [f32; 2] = [400., 150.];
/// Maximum distance between grenade explosion and its target
#[cfg(feature = "test")]
const EXPLOSION_TOLERANCE_METERS: f32 = 6.;
/// Individual which must move to throw, must move at least this distance
#[cfg(feature = "test")]
const TGAM_MIN_MOVE_METERS: f32 = 10.;

const TGWEN_POS: [f32; 2] = [150., 150.];
/// Enemy starts this far (meters) behind the distance under which grenade is thrown
const TGWEN_ENEMY_MARGIN_METERS: f32 = 5.;

const MSA_LEADER_POS: [f32; 2] = [150., 150.];
const MSA_POS1: [f32; 2] = [180., 150.];
const MSA_POS2: [f32; 2] = [210., 150.];

const MSAO_LEADER_POS: [f32; 2] = [160., 415.];
const MSAO_POS1: [f32; 2] = [235., 415.];
const MSAO_POS2: [f32; 2] = [250., 415.];

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());

    #[cfg(not(feature = "test"))]
    {
        if args.test {
            panic!("Using --test imply enable test feature")
        }
    }

    let setup = match args.case {
        TestCase::Idle => {
            vec![(IDLE_LEADER_POS, vec![])]
        }
        TestCase::Stunned => {
            vec![(STUNNED_POS, vec![])]
        }
        TestCase::ThrowGrenade | TestCase::ThrowGrenadeAfterMove => {
            let target = args.case.throw_target().unwrap();
            vec![(
                TG_LEADER_POS,
                vec![Order::Grenade(target.into()); args.grenade_count],
            )]
        }
        TestCase::ThrowGrenadeWhenEnnemyNear => {
            vec![(TGWEN_POS, vec![])]
        }
        TestCase::MoveStraightAhead => {
            vec![(
                MSA_LEADER_POS,
                vec![
                    Order::MoveTo(MSA_POS1.into()),
                    Order::MoveTo(MSA_POS2.into()),
                ],
            )]
        }
        TestCase::MoveStraightAheadObstacle => {
            vec![(
                MSAO_LEADER_POS,
                vec![
                    Order::MoveTo(MSAO_POS1.into()),
                    Order::MoveTo(MSAO_POS2.into()),
                ],
            )]
        }
        TestCase::MoveFastStraightAhead => {
            vec![(
                MSA_LEADER_POS,
                vec![
                    Order::MoveFastTo(MSA_POS1.into()),
                    Order::MoveFastTo(MSA_POS2.into()),
                ],
            )]
        }
        TestCase::MoveFastStraightAheadObstacle => {
            vec![(
                MSAO_LEADER_POS,
                vec![
                    Order::MoveFastTo(MSAO_POS1.into()),
                    Order::MoveFastTo(MSAO_POS2.into()),
                ],
            )]
        }
    };

    let enemies = match args.case {
        TestCase::ThrowGrenadeWhenEnnemyNear => vec![ennemy_near_start()],
        _ => vec![],
    };

    let run = behavior::run()
        .test(args.test)
        .setup(setup)
        .enemies(enemies)
        .individual_count(args.individual_count)
        .grenade_count(args.grenade_count);

    let run = {
        #[cfg_attr(not(feature = "test"), allow(clippy::let_unit_value))]
        let test_track = {
            #[cfg(feature = "test")]
            {
                use oc_world_server::tracker::Tracker;

                let args = args.clone();
                let count = args.individual_count;
                Box::new(move |tracker: Tracker| match args.case {
                    // FIXME: test if squad member reach expected position
                    TestCase::Idle | TestCase::Stunned => {}
                    TestCase::ThrowGrenade | TestCase::ThrowGrenadeAfterMove => {
                        assert_throw(&tracker.take(), &args);
                        println!("✅ (SERVER) All tests passed");
                    }
                    TestCase::ThrowGrenadeWhenEnnemyNear => {
                        assert_thrown(&tracker.take(), &args);
                        println!("✅ (SERVER) All tests passed");
                    }
                    TestCase::MoveStraightAhead
                    | TestCase::MoveStraightAheadObstacle
                    | TestCase::MoveFastStraightAhead
                    | TestCase::MoveFastStraightAheadObstacle => {
                        let tracker = tracker.take();

                        for i in 0..count as u64 {
                            let accomplished = (
                                oc_individual::IndividualIndex(i),
                                oc_individual::Update::Accomplished,
                            );
                            assert!(
                                tracker.individuals.contains(&accomplished),
                                "Individual {i} did not accomplish its orders"
                            );
                        }

                        println!("✅ (SERVER) All tests passed");
                    }
                })
            }
            #[cfg(not(feature = "test"))]
            {}
        };

        run.tests((
            {
                let args_ = args.clone();
                Box::new(move |app| {
                    app.insert_resource(Args_(args_.clone()));

                    #[cfg(feature = "debug")]
                    app.add_systems(Startup, |mut commands: Commands| {
                        commands.trigger(ToggleShowFormationPositions)
                    });
                    #[cfg(feature = "debug")]
                    if matches!(args.case, TestCase::Stunned) {
                        app.add_systems(Update, stunned_fire);
                    }
                    #[cfg(feature = "test")]
                    {
                        match args.case {
                            TestCase::Idle => {}
                            TestCase::Stunned => {
                                app.init_resource::<StunnedTest>()
                                    .add_observer(on_stunned_set_status)
                                    .add_observer(on_stunned_set_intent)
                                    .add_systems(Update, stunned_test);
                            }
                            TestCase::ThrowGrenade
                            | TestCase::ThrowGrenadeAfterMove
                            | TestCase::ThrowGrenadeWhenEnnemyNear => {
                                app.init_resource::<Tracking>();
                            }
                            TestCase::MoveStraightAhead
                            | TestCase::MoveStraightAheadObstacle
                            | TestCase::MoveFastStraightAhead
                            | TestCase::MoveFastStraightAheadObstacle => {
                                app.init_resource::<Tracking>()
                                    .add_observer(on_move_step_accomplished_event)
                                    .add_observer(on_accomplished_event);
                            }
                        }
                        if !matches!(args.case, TestCase::Stunned) {
                            app.add_systems(Update, end_when_success_or_timeout);
                        }
                    }
                })
            },
            test_track,
        ))
    };

    run.call()?;

    Ok(())
}

#[allow(unused)]
#[derive(Debug, Resource)]
struct Args_(Args);

#[cfg(feature = "test")]
#[derive(Debug, Resource, Default)]
struct Tracking {
    move_step_accomplished: Vec<MoveStepAccomplishedEvent>,
    accomplished: Vec<AccomplishedEvent>,
}

#[cfg(feature = "test")]
use {
    oc_battle_gui::states::Game, oc_root::Wcfg, std::sync::Mutex, std::time::Duration,
    std::time::Instant,
};

/// Enemy start position and order: it walks to the idle individual from out of grenade distance.
fn ennemy_near_start() -> ([f32; 2], Vec<Order>) {
    let w = oc_root::WorldConfig::new(1, 1);
    let margin = oc_root::physics::Meters(TGWEN_ENEMY_MARGIN_METERS);
    let distance = w.individual_grenade_enemy_distance().pixels(&w) + margin.pixels(&w);
    let start = [TGWEN_POS[0] + distance, TGWEN_POS[1]];
    (start, vec![Order::MoveTo(TGWEN_POS.into())])
}

#[cfg(feature = "test")]
fn end_when_success_or_timeout(
    mut commands: Commands,
    game: Res<Game>,
    args: Res<Args_>,
    tracking: Res<Tracking>,
    w: Res<Wcfg>,
    individuals: Query<(
        &oc_battle_gui::entity::individual::IndividualIndex,
        &oc_battle_gui::ingame::individual::Weapons,
    )>,
) {
    let_some!(w = &w.0, return);
    static MOVE_DONE: Mutex<Option<Instant>> = Mutex::new(None);
    let timeout = match args.0.case {
        TestCase::Idle | TestCase::Stunned => Duration::from_secs(20),
        TestCase::ThrowGrenade => Duration::from_secs(20),
        TestCase::ThrowGrenadeAfterMove => Duration::from_secs(40),
        TestCase::ThrowGrenadeWhenEnnemyNear => Duration::from_secs(40),
        TestCase::MoveStraightAhead => Duration::from_secs(40),
        TestCase::MoveStraightAheadObstacle => Duration::from_secs(50),
        TestCase::MoveFastStraightAhead => Duration::from_secs(30),
        TestCase::MoveFastStraightAheadObstacle => Duration::from_secs(40),
    };
    let speed = w.speed();
    // Ensure a minimal timeout to let time to bevy to make "network" exchange with server
    // Throwing is not shortened by speed as much as moving: give it more time
    let min_timeout = match args.0.case {
        TestCase::ThrowGrenade
        | TestCase::ThrowGrenadeAfterMove
        | TestCase::ThrowGrenadeWhenEnnemyNear => Duration::from_secs(10),
        _ => Duration::from_secs(2),
    };
    let timeout = timeout.div_f32(speed).max(min_timeout);

    let timeout = game.started.elapsed() > timeout;
    let mut move_done = MOVE_DONE.lock().unwrap();
    *move_done = match *move_done {
        None => match args.0.case {
            TestCase::Idle | TestCase::Stunned => None,
            // All individuals threw their grenades. Server tests explosions.
            TestCase::ThrowGrenade
            | TestCase::ThrowGrenadeAfterMove
            | TestCase::ThrowGrenadeWhenEnnemyNear => {
                // Only individuals with grenades (not the enemies, which are after)
                let empty = individuals
                    .iter()
                    .filter(|(i, weapons)| {
                        (i.0.0 as usize) < args.0.individual_count && weapons.grenades.is_empty()
                    })
                    .count();
                (empty == args.0.individual_count).then(Instant::now)
            }
            TestCase::MoveStraightAhead
            | TestCase::MoveStraightAheadObstacle
            | TestCase::MoveFastStraightAhead
            | TestCase::MoveFastStraightAheadObstacle => {
                {
                    // Leader accomplish its two orders; members' orders are replaced by the
                    // leader's current one, so they must accomplish at least one.
                    let accomplished = |i: u64| {
                        tracking
                            .accomplished
                            .iter()
                            .filter(|e| ***e == oc_individual::IndividualIndex(i))
                            .count()
                    };
                    (accomplished(0) >= 2
                        && (1..args.0.individual_count as u64).all(|i| accomplished(i) >= 1))
                    .then(Instant::now)
                }
            }
        },
        Some(value) => {
            if value.elapsed().as_secs_f32() > 1. / speed {
                println!("✅ (GUI) Individual reached target");
                commands.write_message(bevy::app::AppExit::from_code(0));
            };
            Some(value)
        }
    };

    if timeout {
        eprintln!("❌ Timeout reached ! Individual didn't reached target");
        commands.write_message(bevy::app::AppExit::from_code(1));
    }
}

#[cfg(feature = "test")]
fn on_move_step_accomplished_event(
    event: On<MoveStepAccomplishedEvent>,
    mut tracking: ResMut<Tracking>,
) {
    tracking.move_step_accomplished.push(event.clone());
}

#[cfg(feature = "test")]
fn on_accomplished_event(event: On<AccomplishedEvent>, mut tracking: ResMut<Tracking>) {
    tracking.accomplished.push(event.clone());
}

#[cfg(feature = "test")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum StunnedStep {
    /// Waiting explosion effect
    #[default]
    Stunned,
    /// Waiting stun end
    Operational,
    /// Waiting the move order to be followed
    Moving,
}

#[cfg(feature = "test")]
#[derive(Debug, Resource, Default)]
struct StunnedTest {
    step: StunnedStep,
    /// Test result, once known (true for success)
    result: Option<bool>,
}

/// Explosion and move order are sent at the same time, once individual is in game.
#[cfg(feature = "debug")]
fn stunned_fire(
    mut commands: Commands,
    mut fired: Local<bool>,
    w: Res<oc_root::Wcfg>,
    individuals: Query<&oc_battle_gui::entity::individual::IndividualIndex>,
) {
    use oc_battle_gui::network::output::ToServerEvent;
    use oc_individual::squad::SquadIndex;
    use oc_network::{SquadMessage, ToServer};
    use oc_root::geo::WorldVec2;

    let_some!(w = &w.0, return);
    if *fired
        || !individuals
            .iter()
            .any(|i| i.0 == oc_individual::IndividualIndex(0))
    {
        return;
    }
    *fired = true;

    let position = WorldVec2::new(
        STUNNED_POS[0] + oc_root::physics::Meters(STUNNED_EXPLOSION_DISTANCE).pixels(w),
        STUNNED_POS[1],
    );
    commands.trigger(ToServerEvent(ToServer::DebugExplosion(
        STUNNED_SHELL_INDEX,
        position,
    )));
    commands.trigger(ToServerEvent(ToServer::Squad(
        SquadIndex(0),
        SquadMessage::SetOrders(vec![Order::MoveTo(STUNNED_MOVE_TARGET.into())]),
    )));
}

/// Individual must be stunned, then operational, then moving. Steps are followed from received
/// updates (and not from components state at each frame): stun can be shorter than a frame (with
/// fast speed and slow rendering), so stunned then operational status can be received in one frame.
#[cfg(feature = "test")]
fn on_stunned_set_status(
    event: On<oc_battle_gui::ingame::individual::SetStatusEvent>,
    mut test: ResMut<StunnedTest>,
) {
    if event.0 != oc_individual::IndividualIndex(0) || test.result.is_some() {
        return;
    }

    match (test.step, event.1) {
        (StunnedStep::Stunned, oc_individual::Status::Stunned(_)) => {
            println!("✅ (GUI) Individual stunned");
            test.step = StunnedStep::Operational;
        }
        (StunnedStep::Operational, oc_individual::Status::Operational) => {
            println!("✅ (GUI) Individual operational again");
            test.step = StunnedStep::Moving;
        }
        _ => {}
    }
}

#[cfg(feature = "test")]
fn on_stunned_set_intent(
    event: On<oc_battle_gui::ingame::individual::SetIntentEvent>,
    mut test: ResMut<StunnedTest>,
) {
    use oc_individual::behavior::Intent;

    if event.0 != oc_individual::IndividualIndex(0)
        || test.result.is_some()
        || !matches!(event.1, Intent::MoveTo(..))
    {
        return;
    }

    if test.step == StunnedStep::Moving {
        println!("✅ (GUI) Individual moving");
        test.result = Some(true);
    } else {
        eprintln!("❌ Individual moves before ({:?})", test.step);
        test.result = Some(false);
    }
}

#[cfg(feature = "test")]
fn stunned_test(mut commands: Commands, test: Res<StunnedTest>, game: Res<Game>, w: Res<Wcfg>) {
    let_some!(w = &w.0, return);
    match test.result {
        Some(true) => {
            commands.write_message(bevy::app::AppExit::from_code(0));
            return;
        }
        Some(false) => {
            commands.write_message(bevy::app::AppExit::from_code(1));
            return;
        }
        None => {}
    }

    let timeout = Duration::from_secs(20)
        .div_f32(w.speed())
        .max(Duration::from_secs(5));
    if game.started.elapsed() > timeout {
        eprintln!("❌ Timeout reached ! Step {:?} not reached", test.step);
        commands.write_message(bevy::app::AppExit::from_code(1));
    }
}

/// Every individual must validate the throw, and explosions must match grenades count and target.
#[cfg(feature = "test")]
fn assert_throw(tracker: &oc_world_server::tracker::Tracker_, args: &Args) {
    use oc_root::{WorldConfig, physics::Meters};

    let w = WorldConfig::new(1, 1);
    let target = args.case.throw_target().unwrap();
    let moves = matches!(args.case, TestCase::ThrowGrenadeAfterMove);

    let validated = (0..args.individual_count as u64)
        .map(oc_individual::IndividualIndex)
        .filter(|i| {
            no_more_grenade(tracker, *i) && (!moves || moved_in_range(tracker, &w, *i, target))
        })
        .count();
    assert_eq!(
        validated, args.individual_count,
        "Only {validated} individuals on {} validate the throw",
        args.individual_count
    );

    assert_explosions_count(tracker, args);

    // Grenade lands around the order position (see throw scatter)
    let tolerance = Meters(EXPLOSION_TOLERANCE_METERS).pixels(&w);
    for (position, _) in &tracker.explosions {
        assert!(
            Vec2::from(*position).distance(target.into()) <= tolerance,
            "Explosion at {position:?} is too far from target {target:?}"
        );
    }
}

/// Every individual threw all its grenades: it is not asked to throw on a given target.
#[cfg(feature = "test")]
fn assert_thrown(tracker: &oc_world_server::tracker::Tracker_, args: &Args) {
    let thrown = (0..args.individual_count as u64)
        .map(oc_individual::IndividualIndex)
        .filter(|i| no_more_grenade(tracker, *i))
        .count();
    assert_eq!(
        thrown, args.individual_count,
        "Only {thrown} individuals on {} threw their grenades",
        args.individual_count
    );
    assert_explosions_count(tracker, args);
}

/// One explosion by grenade
#[cfg(feature = "test")]
fn assert_explosions_count(tracker: &oc_world_server::tracker::Tracker_, args: &Args) {
    assert_eq!(
        tracker.explosions.len(),
        args.individual_count * args.grenade_count,
        "Unexpected explosions count"
    );
}

/// Individual last known weapons own no grenade
#[cfg(feature = "test")]
fn no_more_grenade(
    tracker: &oc_world_server::tracker::Tracker_,
    i: oc_individual::IndividualIndex,
) -> bool {
    tracker
        .individuals
        .iter()
        .filter(|(index, _)| *index == i)
        .filter_map(|(_, update)| match update {
            oc_individual::Update::SetWeapons(weapons) => Some(weapons),
            _ => None,
        })
        .next_back()
        .is_some_and(|weapons| weapons.grenades.is_empty())
}

/// Individual moved a minimal distance and ended in throw range of target
#[cfg(feature = "test")]
fn moved_in_range(
    tracker: &oc_world_server::tracker::Tracker_,
    w: &oc_root::WorldConfig,
    i: oc_individual::IndividualIndex,
    target: [f32; 2],
) -> bool {
    use oc_root::{geo::WorldVec3, physics::Meters};

    let mut positions = tracker
        .individual_positions
        .iter()
        .filter(|(index, _)| *index == i)
        .map(|(_, position)| *position);
    let Some(first) = positions.next() else {
        return false;
    };
    let last = positions.next_back().unwrap_or(first);

    let target = WorldVec3::new(target[0], target[1], last.z);
    first.distance(last) > Meters(TGAM_MIN_MOVE_METERS).pixels(w)
        && last.distance(target) <= oc_individual::behavior::THROW_MAX_DISTANCE.pixels(w)
}
