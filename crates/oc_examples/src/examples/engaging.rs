#![allow(clippy::unwrap_used)]

use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
#[cfg(feature = "test")]
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use bevy::prelude::*;

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
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
    battle::BattlePhase,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    side,
    utils::Frequency,
};
use oc_utils::d2::Direction;
#[cfg(feature = "test")]
use oc_utils::{let_some, true_};
use oc_world::{meta::Meta, tile::Tile};
use tests::{
    individual::TestIndividual,
    squad::TestSquad,
    weapons::{TestWeapon, TestWeapons},
};

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

// FIXME BS NOW: seems target not correct pixel (top left instead center ?)
// il faut changer le système pour considérer le pixel de position d'un individual comme son centre et plus son to left
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

// FIXME BS NOW: set random precision (fire) and permit set precise at 100% for test
pub fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());
    if args.test {
        #[cfg(any(not(feature = "test"), not(feature = "debug")))]
        {
            panic!("To enable test, feature `test` and `debug` must be enabled too")
        }
    }

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
    let mod__ = oc_mod::Mod::load(&mod_, None)?;
    let map = PathBuf::from("maps/meadow1");
    let meta = Meta::from_file(&map.join("meta.toml"))?;
    let map_ = oc_world::reader::MapReader::new(&map);
    let map_ = map_.context(format!("Read map_ {}", map.display()))?;
    let w = WorldConfig::new(map_.width().unwrap() as u64, map_.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z))
        .with_visibilities_tick(Frequency::new(2.0)); // To ensure one shot and test it

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

    let tiles = map_.tiles(&w, &mod__).unwrap();

    let individuals = individuals(&w, &mod__, &tiles, &args);
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
        .snapshot(snapshot)
        .test_app_exit_code(args.test);

    #[cfg(feature = "test")]
    let example = example.install(Box::new(install));

    example.build().run()?;

    if args.test {
        if !SUCCESS.load(Ordering::Relaxed) {
            anyhow::bail!("❌ Test failed")
        }
        println!("✅ Test success !");
    }

    Ok(())
}

fn individuals(
    w: &WorldConfig,
    mod_: &oc_mod::Mod,
    _tiles: &Vec<Tile>,
    args: &Args,
) -> Vec<oc_individual::Individual> {
    match args.case {
        TestCase::Direct => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 250., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Weapon3").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(250., 150., 0.))
                .build()
                .make(w),
        ],
        TestCase::Direct2 => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 250., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Weapon3").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(250., 150., 0.))
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(260., 150., 0.))
                .build()
                .make(w),
        ],
        TestCase::EngageOneByOne => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 250., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Weapon3").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(250., 150., 0.))
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(260., 150., 0.))
                .build()
                .make(w),
        ],
        TestCase::FarMachineGun => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(350., 450., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "FullFast").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(50., 50., 0.))
                .build()
                .make(w),
        ],
        TestCase::Suppressed => [
            vec![
                TestIndividual::builder()
                    .side(side::Side::A)
                    .position(WorldVec3::new(104., 311., 0.))
                    .weapons(
                        TestWeapons::builder()
                            .primary(TestWeapon::filled(mod_, "Weapon3").make())
                            .build()
                            .make(),
                    )
                    .build()
                    .make(w),
            ],
            (0..10)
                .map(|i| {
                    TestIndividual::builder()
                        .side(side::Side::B)
                        .position(WorldVec3::new(387., 160. + i as f32 * 30., 0.))
                        .weapons(
                            TestWeapons::builder()
                                .primary(TestWeapon::filled(mod_, "FullFast").make())
                                .build()
                                .make(),
                        )
                        .build()
                        .make(w)
                })
                .collect::<Vec<_>>(),
        ]
        .concat(),
        TestCase::SuppressFire => {
            let magazine = mod_
                .magazines
                .iter()
                .find(|m| m.name() == "Magazine1")
                .unwrap()
                .index();
            vec![
                TestIndividual::builder()
                    .side(side::Side::A)
                    .position(WorldVec3::new(104., 311., 0.))
                    .weapons(
                        TestWeapons::builder()
                            .primary(TestWeapon::filled(mod_, "Weapon3").make())
                            .build()
                            .make(),
                    )
                    .magazines(vec![magazine; SUPPRESS_FIRE_MAGAZINES])
                    .build()
                    .make(w),
            ]
        }
        TestCase::Hedge => {
            vec![
                TestIndividual::builder()
                    .side(side::Side::A)
                    .position(WorldVec3::new(104., 311., 0.))
                    .build()
                    .make(w),
                TestIndividual::builder()
                    .side(side::Side::B)
                    .position(WorldVec3::new(387., 160., 0.))
                    .weapons(
                        TestWeapons::builder()
                            .primary(TestWeapon::filled(mod_, "Weapon3").make())
                            .build()
                            .make(),
                    )
                    .build()
                    .make(w),
            ]
        }
        TestCase::MoveToHiding => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(119., 92., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Mosin Nagant M1924").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(119., 99., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Mosin Nagant M1924").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(389., 64., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Mosin Nagant M1924").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(389., 71., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Mosin Nagant M1924").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
        ],
        TestCase::MoveToHidingThenDiscover => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(119., 92., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Mosin Nagant M1924").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(119., 99., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Mosin Nagant M1924").make())
                        .build()
                        .make(),
                )
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(389., 64., 0.))
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(389., 71., 0.))
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
        TestCase::Direct => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(1).unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(1)])
                .orders(vec![])
                .build()
                .make(),
        ],
        TestCase::Direct2 => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(1).unwrap().position.into())
                .members(vec![
                    oc_individual::IndividualIndex(1),
                    oc_individual::IndividualIndex(2),
                ])
                .orders(vec![])
                .build()
                .make(),
        ],
        TestCase::EngageOneByOne => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::Engage(oc_individual::IndividualIndex(1))])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(1).unwrap().position.into())
                .members(vec![
                    oc_individual::IndividualIndex(1),
                    oc_individual::IndividualIndex(2),
                ])
                .orders(vec![])
                .build()
                .make(),
        ],
        TestCase::FarMachineGun => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(1).unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(1)])
                .orders(vec![])
                .build()
                .make(),
        ],
        TestCase::Suppressed => [
            vec![
                TestSquad::builder()
                    .position(individuals.first().unwrap().position.into())
                    .members(vec![oc_individual::IndividualIndex(0)])
                    .orders(vec![])
                    .build()
                    .make(),
            ],
            (0..10)
                .map(|i| {
                    TestSquad::builder()
                        .position(individuals.get(i + 1).unwrap().position.into())
                        .members(vec![oc_individual::IndividualIndex(i as u64 + 1)])
                        .orders(vec![])
                        .build()
                        .make()
                })
                .collect::<Vec<_>>(),
        ]
        .concat(),
        TestCase::SuppressFire => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::Suppress(WorldVec2::new(387., 160.))])
                .build()
                .make(),
        ],
        TestCase::Hedge => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(1).unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(1)])
                .orders(vec![])
                .build()
                .make(),
        ],
        TestCase::MoveToHiding => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![
                    oc_individual::IndividualIndex(0),
                    oc_individual::IndividualIndex(1),
                ])
                .orders(vec![Order::MoveTo(WorldVec2::new(458., 70.))])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(2).unwrap().position.into())
                .members(vec![
                    oc_individual::IndividualIndex(2),
                    oc_individual::IndividualIndex(3),
                ])
                .orders(vec![Order::Hide(Direction::WEST)])
                .build()
                .make(),
        ],
        TestCase::MoveToHidingThenDiscover => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![
                    oc_individual::IndividualIndex(0),
                    oc_individual::IndividualIndex(1),
                ])
                .orders(vec![Order::MoveFastTo(WorldVec2::new(395., 68.))])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(2).unwrap().position.into())
                .members(vec![
                    oc_individual::IndividualIndex(2),
                    oc_individual::IndividualIndex(3),
                ])
                .orders(vec![Order::Hide(Direction::WEST)])
                .build()
                .make(),
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
            // FIXME BS NOW: all individuals visible during some milliseconds
            // default enemy visibility should be hidden
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
