#![allow(clippy::unwrap_used)]

use std::{path::PathBuf, sync::atomic::AtomicBool};
#[cfg(feature = "test")]
use std::{
    sync::{Mutex, atomic::Ordering},
    time::{Duration, Instant},
};

use bevy::prelude::*;

use crate::{
    logging, run,
    snapshot::SnapshotBuilder,
    utils::{self, Loaded, armed_soldier, soldier, squad},
};
use clap::{Parser, ValueEnum};
#[cfg(feature = "test")]
use oc_battle_gui::{
    entity::individual::IndividualIndex,
    ingame::{
        individual::{
            Gesture, SetBehaviorEvent, SetGestureEvent, SetMagazinesEvent, SetStatusEvent,
            SetWeaponsEvent, Status,
        },
        input::projectile::InsertProjectileEvent,
    },
    states::Game,
};
use oc_individual::order::Order;
#[cfg(feature = "test")]
use oc_root::Wcfg;
use oc_root::{
    WorldConfig,
    geo::{WorldVec2, WorldVec3},
    side::Side,
    utils::Frequency,
};
use oc_utils::d2::Direction;
#[cfg(feature = "test")]
use oc_utils::{let_some, true_};
use tests::individual::TestIndividual;

const SUPPRESS_FIRE_MAGAZINES: usize = 2;

#[cfg(feature = "test")]
const AFTER_SUCCESS_WAIT: Duration = Duration::from_secs(1);

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,

    #[arg(long, action)]
    test: bool,

    /// All shots are precise (true when --test)
    #[arg(long, action)]
    precise: bool,
}

#[cfg(feature = "test")]
impl Args {
    fn timeout(&self) -> Duration {
        match self.case {
            TestCase::Direct
            | TestCase::Direct2
            | TestCase::EngageOneByOne
            | TestCase::FarMachineGun
            | TestCase::Hedge => Duration::from_secs(10),
            TestCase::Suppressed | TestCase::SuppressFire => Duration::from_secs(45),
            TestCase::MoveToHiding | TestCase::MoveToHidingThenDiscover => Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    Direct,
    Direct2,
    EngageOneByOne,
    FarMachineGun,
    Hedge,
    Suppressed,
    SuppressFire,
    MoveToHiding,
    MoveToHidingThenDiscover,
}

pub fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());
    utils::ensure_test_debug_features(args.test);

    let mod_ = match args.case {
        TestCase::Direct
        | TestCase::Direct2
        | TestCase::EngageOneByOne
        | TestCase::FarMachineGun
        | TestCase::Hedge
        | TestCase::Suppressed
        | TestCase::SuppressFire => PathBuf::from("mods/tests1"),
        TestCase::MoveToHiding | TestCase::MoveToHidingThenDiscover => PathBuf::from("mods/std1"),
    };
    let map = PathBuf::from("maps/meadow1");
    let Loaded {
        mod_: mod__,
        map: map_,
        w,
        ..
    } = Loaded::new(&mod_, &map)?;
    let w = w.with_visibilities_tick(Frequency::new(2.0)); // To ensure one shot and test it

    // For som cases, apply perfect accuracy
    #[cfg(feature = "debug")]
    if args.test || args.precise {
        WorldConfig::set_inaccuracy_spread_enabled(true);
        WorldConfig::set_inaccuracy_spread(0.0);
    }

    // For some case (and when test) increase the individual tick rate
    let w = match args.case {
        TestCase::Direct
        | TestCase::Direct2
        | TestCase::EngageOneByOne
        | TestCase::MoveToHiding
        | TestCase::MoveToHidingThenDiscover => w,
        TestCase::FarMachineGun
        | TestCase::Suppressed
        | TestCase::SuppressFire
        | TestCase::Hedge => match args.test {
            true => w.with_individual_tick(Frequency::new(10.0)),
            false => w,
        },
    };

    // For some case, enable the immortal settings
    #[cfg(feature = "debug")]
    match args.case {
        TestCase::Direct
        | TestCase::Direct2
        | TestCase::EngageOneByOne
        | TestCase::FarMachineGun
        | TestCase::Hedge
        | TestCase::SuppressFire
        | TestCase::MoveToHiding
        | TestCase::MoveToHidingThenDiscover => {}
        TestCase::Suppressed => {
            WorldConfig::set_immortality(true);
        }
    };

    let individuals = individuals(&w, &mod__, &args);
    let squads = squads(&individuals, &args);
    let snapshot = SnapshotBuilder::fight(map_, individuals, squads).build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .snapshot(snapshot)
        .test_app_exit_code(args.test);

    #[cfg(feature = "test")]
    let example = example.install(Box::new(install));

    example.build().run()?;

    if args.test {
        utils::test_result(&SUCCESS)?;
    }

    Ok(())
}

fn individuals(w: &WorldConfig, mod_: &oc_mod::Mod, args: &Args) -> Vec<oc_individual::Individual> {
    let armed = |side, position: [f32; 3], weapon| armed_soldier(w, mod_, side, position, weapon);
    const MOSIN: &str = "Mosin Nagant M1924";

    match args.case {
        TestCase::Direct => vec![
            armed(Side::A, [250., 250., 0.], "Weapon3"),
            soldier(w, Side::B, [250., 150., 0.]),
        ],
        TestCase::Direct2 | TestCase::EngageOneByOne => vec![
            armed(Side::A, [250., 250., 0.], "Weapon3"),
            soldier(w, Side::B, [250., 150., 0.]),
            soldier(w, Side::B, [260., 150., 0.]),
        ],
        TestCase::FarMachineGun => vec![
            armed(Side::A, [350., 450., 0.], "FullFast"),
            soldier(w, Side::B, [50., 50., 0.]),
        ],
        TestCase::Suppressed => std::iter::once(armed(Side::A, [104., 311., 0.], "Weapon3"))
            .chain((0..10).map(|i| armed(Side::B, [387., 160. + i as f32 * 30., 0.], "FullFast")))
            .collect(),
        TestCase::SuppressFire => {
            let magazine = mod_
                .magazines
                .iter()
                .find(|m| m.name() == "Magazine1")
                .unwrap()
                .index();
            vec![
                TestIndividual::builder()
                    .side(Side::A)
                    .position(WorldVec3::new(104., 311., 0.))
                    .weapons(utils::armed(mod_, "Weapon3"))
                    .magazines(vec![magazine; SUPPRESS_FIRE_MAGAZINES])
                    .build()
                    .make(w),
            ]
        }
        TestCase::Hedge => vec![
            soldier(w, Side::A, [104., 311., 0.]),
            armed(Side::B, [387., 160., 0.], "Weapon3"),
        ],
        TestCase::MoveToHiding => vec![
            armed(Side::A, [119., 92., 0.], MOSIN),
            armed(Side::A, [119., 99., 0.], MOSIN),
            armed(Side::B, [389., 64., 0.], MOSIN),
            armed(Side::B, [389., 71., 0.], MOSIN),
        ],
        TestCase::MoveToHidingThenDiscover => vec![
            armed(Side::A, [119., 92., 0.], MOSIN),
            armed(Side::A, [119., 99., 0.], MOSIN),
            soldier(w, Side::B, [389., 64., 0.]),
            soldier(w, Side::B, [389., 71., 0.]),
        ],
    }
}

fn squads(
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        TestCase::Direct | TestCase::FarMachineGun | TestCase::Hedge => vec![
            squad(individuals, &[0], vec![]),
            squad(individuals, &[1], vec![]),
        ],
        TestCase::Direct2 => vec![
            squad(individuals, &[0], vec![]),
            squad(individuals, &[1, 2], vec![]),
        ],
        TestCase::EngageOneByOne => vec![
            squad(
                individuals,
                &[0],
                vec![Order::Engage(oc_individual::IndividualIndex(1))],
            ),
            squad(individuals, &[1, 2], vec![]),
        ],
        TestCase::Suppressed => (0..=10).map(|i| squad(individuals, &[i], vec![])).collect(),
        TestCase::SuppressFire => vec![squad(
            individuals,
            &[0],
            vec![Order::Suppress(WorldVec2::new(387., 160.))],
        )],
        TestCase::MoveToHiding => vec![
            squad(
                individuals,
                &[0, 1],
                vec![Order::MoveTo(WorldVec2::new(458., 70.))],
            ),
            squad(individuals, &[2, 3], vec![Order::Hide(Direction::WEST)]),
        ],
        TestCase::MoveToHidingThenDiscover => vec![
            squad(
                individuals,
                &[0, 1],
                vec![Order::MoveFastTo(WorldVec2::new(395., 68.))],
            ),
            squad(individuals, &[2, 3], vec![Order::Hide(Direction::WEST)]),
        ],
    }
}

static SUCCESS: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "test")]
#[derive(Debug, Resource, Default)]
struct State {
    success: Option<Instant>,
    start: Option<Instant>,
    last_shoots: Vec<(oc_individual::IndividualIndex, Instant)>,
    gestures: Vec<(oc_individual::IndividualIndex, oc_individual::Gesture)>,
    statuses: Vec<(oc_individual::IndividualIndex, oc_individual::Status)>,
    behaviors: Vec<(
        oc_individual::IndividualIndex,
        oc_individual::behavior::Behavior,
    )>,
    filled_counts: Vec<u16>,
    magazine_counts: Vec<usize>,
}

#[cfg(feature = "test")]
fn install(app: &mut bevy::app::App) {
    let args = Args::parse_from(crate::args());

    #[cfg(feature = "test")]
    if args.test {
        app.add_systems(Update, test_tracker)
            .add_systems(Startup, setup)
            .add_observer(on_set_status)
            .add_observer(on_set_gesture)
            .add_observer(on_set_behavior)
            .add_observer(on_set_magazines);
    }

    #[cfg(feature = "test")]
    app.init_resource::<State>();

    match args.case {
        TestCase::Direct
        | TestCase::Direct2
        | TestCase::EngageOneByOne
        | TestCase::FarMachineGun
        | TestCase::Suppressed
        | TestCase::MoveToHiding
        | TestCase::MoveToHidingThenDiscover
        | TestCase::SuppressFire
        | TestCase::Hedge => {
            #[cfg(feature = "test")]
            app.add_systems(Update, (tracking,))
                .add_observer(on_spawn_projectile)
                .add_observer(on_set_weapons);
        }
    }
}

#[cfg(feature = "test")]
fn on_set_weapons(event: On<SetWeaponsEvent>, mut state: ResMut<State>) {
    true_!(event.0 == oc_individual::IndividualIndex(0), return);
    let_some!(primary = &event.1.primary, return);
    state.filled_counts.push(primary.filled_count);
}

#[cfg(feature = "test")]
fn on_set_magazines(event: On<SetMagazinesEvent>, mut state: ResMut<State>) {
    true_!(event.0 == oc_individual::IndividualIndex(0), return);
    state.magazine_counts.push(event.1.len());
}

#[cfg(feature = "test")]
fn on_set_status(event: On<SetStatusEvent>, mut state: ResMut<State>) {
    state.statuses.push((event.0, event.1));
}

#[cfg(feature = "test")]
fn on_set_behavior(event: On<SetBehaviorEvent>, mut state: ResMut<State>) {
    state.behaviors.push((event.0, event.1.clone()));
}

#[cfg(feature = "test")]
fn on_set_gesture(event: On<SetGestureEvent>, mut state: ResMut<State>) {
    state.gestures.push((event.0, event.1.clone()));
}

#[cfg(feature = "test")]
fn test_tracker(mut commands: Commands, game: Res<Game>, state: ResMut<State>, w: Res<Wcfg>) {
    let args = Args::parse_from(crate::args());
    let speed = w.0.as_ref().map(|w| w.speed()).unwrap_or(1.0);
    // Success is detected as soon as it happens: a large minimal timeout does not slow down tests,
    // and let time to (maybe slow) rendering and "network" exchange with server
    let timeout = args.timeout().div_f32(speed).max(Duration::from_secs(30));
    if game.started.elapsed() > timeout
        || state
            .success
            .map(|success| success.elapsed() > AFTER_SUCCESS_WAIT)
            .unwrap_or_default()
    {
        commands.write_message(bevy::app::AppExit::from_code(0));
    }
}

#[cfg(feature = "test")]
fn setup(mut state: ResMut<State>) {
    state.start = Some(Instant::now());
}

#[cfg(feature = "test")]
fn on_spawn_projectile(projectile: On<InsertProjectileEvent>, mut state: ResMut<State>) {
    if let Some(shooter) = projectile.1.shooter() {
        state.last_shoots.push((shooter, Instant::now()));
    }
}

#[cfg(feature = "test")]
fn tracking(
    mut state: ResMut<State>,
    query: Query<(&IndividualIndex, &Status, &Gesture, &Visibility)>,
    w: Res<Wcfg>,
) {
    let_some!(w = &w.0, return);
    let speed = w.speed();
    let args = Args::parse_from(crate::args());

    static I0_SEEN_PRONE: AtomicBool = AtomicBool::new(false);
    static I0_SEEN_DEAD: AtomicBool = AtomicBool::new(false);
    static I1_SEEN_DEAD: AtomicBool = AtomicBool::new(false);
    static I2_SEEN_DEAD: AtomicBool = AtomicBool::new(false);
    static I3_SEEN_DEAD: AtomicBool = AtomicBool::new(false);
    static I2_AND_I3_HIDDEN_SINCE: Mutex<Option<Instant>> = Mutex::new(None);

    let i2_visibility = query
        .iter()
        .filter_map(|(i, _, _, visibility)| {
            (i.0 == oc_individual::IndividualIndex(2)).then_some(*visibility)
        })
        .next();
    let i3_visibility = query
        .iter()
        .filter_map(|(i, _, _, visibility)| {
            (i.0 == oc_individual::IndividualIndex(3)).then_some(*visibility)
        })
        .next();

    if state.gestures.iter().any(|(i, gesture)| {
        *i == oc_individual::IndividualIndex(0)
            && matches!(gesture.body, oc_individual::BodyGesture::Prone(_))
    }) {
        I0_SEEN_PRONE.store(true, Ordering::Relaxed);
    }
    if state.statuses.iter().any(|(i, status)| {
        *i == oc_individual::IndividualIndex(0) && *status == oc_individual::Status::Dead
    }) {
        I0_SEEN_DEAD.store(true, Ordering::Relaxed);
    }
    if state.statuses.iter().any(|(i, status)| {
        *i == oc_individual::IndividualIndex(1) && *status == oc_individual::Status::Dead
    }) {
        I1_SEEN_DEAD.store(true, Ordering::Relaxed);
    }
    if state.statuses.iter().any(|(i, status)| {
        *i == oc_individual::IndividualIndex(2) && *status == oc_individual::Status::Dead
    }) {
        I2_SEEN_DEAD.store(true, Ordering::Relaxed);
    }
    if state.statuses.iter().any(|(i, status)| {
        *i == oc_individual::IndividualIndex(3) && *status == oc_individual::Status::Dead
    }) {
        I3_SEEN_DEAD.store(true, Ordering::Relaxed);
    }
    let i0_last_shoot = state.last_shoots.last().cloned().and_then(|(i, instant)| {
        match i == oc_individual::IndividualIndex(0) {
            true => Some(instant),
            false => None,
        }
    });
    if matches!(i2_visibility, Some(Visibility::Hidden))
        && matches!(i3_visibility, Some(Visibility::Hidden))
    {
        I2_AND_I3_HIDDEN_SINCE
            .lock()
            .unwrap()
            .get_or_insert_with(Instant::now);
    } else {
        *I2_AND_I3_HIDDEN_SINCE.lock().unwrap() = None;
    }

    if match args.case {
        TestCase::Direct | TestCase::FarMachineGun => {
            I1_SEEN_DEAD.load(Ordering::Relaxed) && I0_SEEN_PRONE.load(Ordering::Relaxed)
        }
        TestCase::Direct2 => {
            I1_SEEN_DEAD.load(Ordering::Relaxed)
                && I2_SEEN_DEAD.load(Ordering::Relaxed)
                && I0_SEEN_PRONE.load(Ordering::Relaxed)
        }
        TestCase::EngageOneByOne => [1, 2].into_iter().all(|target| {
            state.behaviors.iter().any(|(i, behavior)| {
                *i == oc_individual::IndividualIndex(0)
                    && *behavior
                        == oc_individual::behavior::Behavior::Engage(
                            oc_individual::IndividualIndex(target),
                        )
            })
        }),
        TestCase::Suppressed => i0_last_shoot
            .or(state.start)
            .map(|t| t.elapsed().as_secs_f32() > 8. / speed)
            .unwrap_or_default(),
        TestCase::SuppressFire => {
            state.filled_counts == [4, 3, 2, 1, 0, 5, 4, 3, 2, 1, 0, 5, 4, 3, 2, 1, 0]
                && state.magazine_counts == [1, 0]
        }
        TestCase::Hedge => I0_SEEN_DEAD.load(Ordering::Relaxed),
        TestCase::MoveToHiding => I2_AND_I3_HIDDEN_SINCE
            .lock()
            .unwrap()
            .map(|s| s.elapsed().as_secs_f32() > 5. / speed)
            .unwrap_or_default(),
        TestCase::MoveToHidingThenDiscover => {
            state
                .start
                .is_some_and(|s| s.elapsed().as_secs_f32() > 2. / speed)
                && matches!(i2_visibility, Some(Visibility::Visible))
                && matches!(i3_visibility, Some(Visibility::Visible))
        }
    } {
        if state.success.is_none() {
            state.success = Some(Instant::now());
        };

        SUCCESS.store(true, Ordering::Relaxed);
    }
}
