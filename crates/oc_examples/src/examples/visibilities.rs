#![cfg_attr(not(feature = "test"), allow(dead_code, unused_imports))]
#![allow(clippy::unwrap_used)]

use std::time::Duration;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
#[cfg(feature = "test")]
use std::{sync::Mutex, time::Instant};

use bevy::prelude::*;

use crate::{
    logging, run,
    snapshot::SnapshotBuilder,
    utils::{self, Loaded},
};
use clap::{Parser, ValueEnum};
#[cfg(feature = "test")]
use oc_battle_gui::{
    entity::individual::IndividualIndex,
    ingame::individual::{Gesture, SetGestureEvent},
    states::{Game, GameConfig},
};
use oc_individual::order::Order;
#[allow(unused_imports)]
use oc_mod::shell::ShellIndex;
#[cfg(feature = "test")]
use oc_root::Wcfg;
use oc_root::{
    WorldConfig,
    geo::{WorldVec2, WorldVec3},
    side::Side,
    utils::Frequency,
};
use oc_utils::d2::Direction;
#[cfg(any(feature = "test", feature = "debug"))]
use oc_utils::let_some;

#[cfg(feature = "test")]
/// Minimal time to let bevy make "network" exchange with server, whatever the speed. Success is
/// detected as soon as it happens: a large value does not slow down tests, and let time to (maybe
/// slow) rendering
const MIN_TIMEOUT: Duration = Duration::from_secs(30);
const MIN_EXPLOSION_DELAY: Duration = Duration::from_secs(1);
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

#[cfg(feature = "test")]
impl Args {
    fn timeout(&self) -> Duration {
        match self.case {
            TestCase::Direct | TestCase::Through | TestCase::NotVisible | TestCase::Hidden => {
                Duration::from_secs(10)
            }
            TestCase::Discover | TestCase::AlteredWall => Duration::from_secs(20),
            TestCase::MoveThenEnemyVisible => Duration::from_secs(40),
            TestCase::EnnemyVisibleAfterShot => Duration::from_secs(40),
        }
    }
}

// FIXME BS NOW: ennemy hiding at start is visible one milli seconds
#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    Direct,
    Through,
    NotVisible,
    Discover,
    MoveThenEnemyVisible,
    // FIXME BS NOW: implement hiding bonuses:
    //   - visibility when immobile
    //   - visibility when crawling ?
    // FIXME BS NOW 2: when fire on
    // FIXME BS NOW 3: tile nature should sometime (brick wall 90%, trunk 70%, etc) "block" projectile
    Hidden,
    /// Individual hidden behind walls becomes visible once an explosion destroyed them
    AlteredWall,
    EnnemyVisibleAfterShot,
}

const ENNEMY_SUPPRESS_ORDER_DELAY: Duration = Duration::from_secs(5);
const ENNEMY_SUPPRESS_TARGET: [f32; 2] = [470., 198.];
const WEAPON: &str = "Weapon3";
const ENNEMY_MAGAZINES: usize = 4;

#[allow(unused)]
const DESTRUCTION_SHELL: ShellIndex = ShellIndex(3); // "Destruction Only (10 meters)"
#[allow(unused)]
const ALTERED_WALL_EXPLOSION_POSITION: [f32; 2] = [60., 78.];

pub fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());
    utils::ensure_test_feature(args.test);

    let mod_ = PathBuf::from(MOD);
    let map = PathBuf::from("maps/meadow1");
    let loaded = Loaded::new(&mod_, &map)?;
    let w = loaded
        .w
        .with_visibilities_tick(Frequency::each(std::time::Duration::from_millis(100)));

    // Precise shots to ensure kills
    #[cfg(feature = "debug")]
    if args.test && matches!(args.case, TestCase::EnnemyVisibleAfterShot) {
        WorldConfig::set_inaccuracy_spread_enabled(true);
        WorldConfig::set_inaccuracy_spread(0.0);
    }

    let individuals = individuals(&w, &loaded.mod_, &args);
    let squads = squads(&individuals, &args);
    let snapshot =
        SnapshotBuilder::fight(loaded.map, individuals, squads).build(w, &loaded.mod_)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .install(Box::new(install))
        .snapshot(snapshot)
        .test_app_exit_code(args.test);

    example.build().run()?;

    if args.test {
        utils::test_result(&SUCCESS)?;
    }

    Ok(())
}

fn magazine(mod_: &oc_mod::Mod) -> oc_mod::magazine::MagazineIndex {
    mod_.magazines
        .iter()
        .find(|m| m.name() == "Magazine1")
        .unwrap()
        .index()
}

/// One individual by side
fn individuals(w: &WorldConfig, mod_: &oc_mod::Mod, args: &Args) -> Vec<oc_individual::Individual> {
    let (a, b) = match args.case {
        TestCase::Direct => ([250., 250., 0.], [250., 150., 0.]),
        TestCase::Through => ([250., 250., 0.], [450., 250., 0.]),
        TestCase::NotVisible => ([250., 250., 0.], [450., 150., 0.]),
        TestCase::Discover | TestCase::MoveThenEnemyVisible => ([250., 225., 0.], [450., 140., 0.]),
        TestCase::Hidden => ([25., 224., 0.], [387., 198., 0.]),
        TestCase::AlteredWall => ([130., 75., 0.], [33., 78., 0.]),
        TestCase::EnnemyVisibleAfterShot => {
            return vec![
                utils::armed_soldier(w, mod_, Side::A, [120., 224., 0.], WEAPON),
                tests::individual::TestIndividual::builder()
                    .side(Side::B)
                    .position(WorldVec3::new(387., 198., 0.))
                    .weapons(utils::armed(mod_, WEAPON))
                    .magazines(vec![magazine(mod_); ENNEMY_MAGAZINES])
                    .build()
                    .make(w),
            ];
        }
    };

    vec![utils::soldier(w, Side::A, a), utils::soldier(w, Side::B, b)]
}

fn squads(
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    let orders = match args.case {
        TestCase::Direct
        | TestCase::Through
        | TestCase::NotVisible
        | TestCase::Hidden
        | TestCase::AlteredWall => vec![],
        TestCase::Discover => vec![Order::MoveFastTo(WorldVec2::new(250., 110.))],
        TestCase::MoveThenEnemyVisible => vec![Order::MoveTo(WorldVec2::new(250., 100.))],
        TestCase::EnnemyVisibleAfterShot => vec![],
    };
    let ennemy_orders = match args.case {
        TestCase::EnnemyVisibleAfterShot => vec![Order::Hide(Direction::WEST)],
        _ => vec![],
    };
    vec![
        utils::squad(individuals, &[0], orders),
        utils::squad(individuals, &[1], ennemy_orders),
    ]
}

static SUCCESS: AtomicBool = AtomicBool::new(false);
#[allow(unused)]
static EXPLOSION_FIRED: AtomicBool = AtomicBool::new(false);
#[allow(unused)]
static SUPPRESS_ORDERED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "test")]
#[derive(Debug, Resource, Default)]
struct State {
    success: Option<Instant>,
    gestures: Vec<(oc_individual::IndividualIndex, oc_individual::Gesture)>,
    statuses: Vec<(oc_individual::IndividualIndex, oc_individual::Status)>,
}

#[allow(unused)]
fn install(app: &mut bevy::app::App) {
    let args = Args::parse_from(crate::args());

    #[cfg(feature = "test")]
    if args.test {
        app.add_systems(Update, test_tracker);
    }

    #[cfg(feature = "test")]
    app.init_resource::<State>();

    match args.case {
        TestCase::Direct
        | TestCase::Through
        | TestCase::NotVisible
        | TestCase::Discover
        | TestCase::MoveThenEnemyVisible
        | TestCase::Hidden => {
            #[cfg(feature = "test")]
            app.add_systems(Update, tracking)
                .add_observer(on_set_gesture);
        }
        TestCase::AlteredWall => {
            #[cfg(feature = "test")]
            app.add_systems(Update, tracking)
                .add_observer(on_set_gesture);
            #[cfg(feature = "debug")]
            app.add_systems(Update, fire_explosion);
        }
        TestCase::EnnemyVisibleAfterShot => {
            #[cfg(feature = "test")]
            app.add_systems(Update, tracking)
                .add_observer(on_set_gesture)
                .add_observer(on_set_status);
            app.add_systems(Update, order_suppress);
        }
    }
}

/// Suppress order for EnnemyVisibleAfterShot case
fn order_suppress(
    mut commands: Commands,
    mut since: Local<Option<std::time::Instant>>,
    w: Res<oc_root::Wcfg>,
    mut ordered: Local<bool>,
) {
    use oc_battle_gui::network::output::ToServerEvent;
    use oc_individual::squad::SquadIndex;
    use oc_network::{SquadMessage, ToServer};

    let Some(w) = &w.0 else { return };
    if *ordered {
        return;
    }

    let first = *since.get_or_insert_with(std::time::Instant::now);
    if first.elapsed() < ENNEMY_SUPPRESS_ORDER_DELAY.div_f32(w.speed()) {
        return;
    }

    *ordered = true;
    commands.trigger(ToServerEvent(ToServer::Squad(
        SquadIndex(1),
        SquadMessage::SetOrders(vec![Order::Suppress(ENNEMY_SUPPRESS_TARGET.into())]),
    )));
}

fn explosion_delay(speed: f32) -> Duration {
    Duration::from_secs(2)
        .div_f32(speed)
        .max(MIN_EXPLOSION_DELAY)
}

/// Destroy the walls
#[cfg(feature = "debug")]
fn fire_explosion(
    mut commands: Commands,
    mut fired: Local<Option<std::time::Instant>>,
    individuals: Query<&oc_battle_gui::entity::individual::IndividualIndex>,
    g: Res<oc_battle_gui::states::GameConfig>,
) {
    use oc_battle_gui::network::output::ToServerEvent;
    use oc_network::ToServer;

    let_some!(g = &g.0, return);
    if individuals.iter().count() < 2 {
        return;
    }

    // Let some time to observe the initial (hidden) state
    let first = *fired.get_or_insert_with(std::time::Instant::now);
    if first.elapsed() < explosion_delay(g.w.speed())
        || EXPLOSION_FIRED.swap(true, Ordering::Relaxed)
    {
        return;
    }

    commands.trigger(ToServerEvent(ToServer::DebugExplosion(
        DESTRUCTION_SHELL,
        WorldVec2::from(ALTERED_WALL_EXPLOSION_POSITION),
    )));
}

#[cfg(feature = "test")]
fn on_set_gesture(event: On<SetGestureEvent>, mut state: ResMut<State>) {
    state.gestures.push((event.0, event.1.clone()));
}

#[cfg(feature = "test")]
fn on_set_status(
    event: On<oc_battle_gui::ingame::individual::SetStatusEvent>,
    mut state: ResMut<State>,
) {
    state.statuses.push((event.0, event.1));
}

#[cfg(feature = "test")]
fn test_tracker(mut commands: Commands, game: Res<Game>, state: ResMut<State>, w: Res<Wcfg>) {
    let_some!(w = &w.0, return);
    let args = Args::parse_from(crate::args());
    let speed = w.speed();
    // Ensure a minimal timeout to let time to bevy to make "network" exchange with server
    let mut timeout = args.timeout().div_f32(speed).max(MIN_TIMEOUT);
    if matches!(args.case, TestCase::AlteredWall) {
        // Let time to see the effect of the explosion
        timeout = timeout.max(explosion_delay(speed) + Duration::from_secs(2));
    }
    if game.started.elapsed() > timeout
        || state
            .success
            .map(|success| success.elapsed() >= AFTER_SUCCESS_WAIT.div_f32(speed))
            .unwrap_or_default()
    {
        commands.write_message(bevy::app::AppExit::from_code(0));
    }
}

#[cfg(feature = "test")]
fn tracking(
    mut state: ResMut<State>,
    query: Query<(&IndividualIndex, &Visibility, &Gesture)>,
    g: Res<GameConfig>,
) {
    let_some!(g = &g.0, return);
    let args = Args::parse_from(crate::args());

    static I2_INVISIBLE_SINCE: Mutex<Option<Instant>> = Mutex::new(None);
    static I2_HIDDEN_BEFORE_EXPLOSION: AtomicBool = AtomicBool::new(false);
    static I2_VISIBLE_BEFORE_SUPPRESS: AtomicBool = AtomicBool::new(false);

    let i1_visible = query
        .iter()
        .any(|(i, v, _)| i.0 == oc_individual::IndividualIndex(0) && v == Visibility::Visible);
    let i1_gesture = query
        .iter()
        .filter_map(|(i, _, g)| (i.0 == oc_individual::IndividualIndex(0)).then_some(&g.0.body))
        .next();
    let i2_visible = query
        .iter()
        .any(|(i, v, _)| i.0 == oc_individual::IndividualIndex(1) && v == Visibility::Visible);
    let i2_gesture = query
        .iter()
        .filter_map(|(i, _, g)| (i.0 == oc_individual::IndividualIndex(1)).then_some(&g.0.body))
        .next();

    if !i2_visible && I2_INVISIBLE_SINCE.lock().unwrap().is_none() {
        *I2_INVISIBLE_SINCE.lock().unwrap() = Some(Instant::now());
    }

    if i2_visible && !SUPPRESS_ORDERED.load(Ordering::Relaxed) {
        I2_VISIBLE_BEFORE_SUPPRESS.store(true, Ordering::Relaxed);
    }

    if !i2_visible && !EXPLOSION_FIRED.load(Ordering::Relaxed) {
        I2_HIDDEN_BEFORE_EXPLOSION.store(true, Ordering::Relaxed);
    }

    if match args.case {
        TestCase::Direct | TestCase::Through => {
            i1_visible
                && i2_visible
                && matches!(i1_gesture, Some(&oc_individual::BodyGesture::Prone(_)))
                && matches!(i2_gesture, Some(&oc_individual::BodyGesture::Prone(_)))
        }
        TestCase::NotVisible => {
            i1_visible
                && !i2_visible
                && matches!(i1_gesture, Some(&oc_individual::BodyGesture::StandUp(_)))
                && matches!(i2_gesture, Some(&oc_individual::BodyGesture::StandUp(_)))
        }
        TestCase::Discover => {
            i1_visible
                && i2_visible
                && matches!(i1_gesture, Some(&oc_individual::BodyGesture::Prone(_)))
                && matches!(i2_gesture, Some(&oc_individual::BodyGesture::Prone(_)))
        }
        TestCase::MoveThenEnemyVisible => state.gestures.iter().any(|(i, gesture)| {
            *i == oc_individual::IndividualIndex(0)
                && matches!(gesture.body, oc_individual::BodyGesture::Prone(_))
        }),
        TestCase::AlteredWall => {
            i1_visible
                && i2_visible
                && I2_HIDDEN_BEFORE_EXPLOSION.load(Ordering::Relaxed)
                && EXPLOSION_FIRED.load(Ordering::Relaxed)
        }
        TestCase::Hidden => {
            i1_visible
                && !i2_visible
                // FIXME BS NOW: add never been seen (must modify source code to begin in correct gesture)
                && I2_INVISIBLE_SINCE
                    .lock()
                    .unwrap()
                    .is_some_and(|s| s.elapsed().as_secs_f32() >= 1. / g.w.speed())
        }
        TestCase::EnnemyVisibleAfterShot => {
            !I2_VISIBLE_BEFORE_SUPPRESS.load(Ordering::Relaxed)
                && SUPPRESS_ORDERED.load(Ordering::Relaxed)
                && state.statuses.iter().any(|(i, status)| {
                    *i == oc_individual::IndividualIndex(1)
                        && *status == oc_individual::Status::Dead
                })
        }
    } {
        // FIXME: must test individuals behavior/gesture too (hide)
        state.success = Some(Instant::now());
        SUCCESS.store(true, Ordering::Relaxed);
    }
}
