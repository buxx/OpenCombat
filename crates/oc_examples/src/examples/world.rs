#![cfg_attr(not(feature = "test"), allow(dead_code, unused_imports))]
#![allow(clippy::unwrap_used)]

#[cfg(feature = "test")]
use oc_root::WcfgFrom;
use std::path::PathBuf;
#[cfg(feature = "test")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use bevy::prelude::*;
use clap::{Parser, ValueEnum};
#[cfg(feature = "test")]
use oc_battle_gui::fx::FxEvent;
#[cfg(feature = "test")]
use oc_battle_gui::ingame::SetBattlePhase;
use oc_battle_gui::ingame::{GameConfigReceived, camera::move_::CenterCameraOn};
use oc_battle_gui::network::output::ToServerEvent;
use oc_individual::order::Order;
use oc_mod::Mod;
use oc_network::ToServer;
#[cfg(feature = "test")]
use oc_physics::update::bevy::Position;
#[cfg(feature = "test")]
use oc_root::battle::{End, Result as BattleResult, SideResult};
use oc_root::{
    WorldConfig,
    battle::BattlePhase,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    side::Side,
};
#[cfg(feature = "test")]
use oc_utils::{d2::AlmostEqual, let_some, true_};
use oc_world::{
    meta::Meta,
    support::{Support, SupportType, Supports},
};
use tests::{
    individual::TestIndividual,
    squad::TestSquad,
    weapons::{TestWeapon, TestWeapons},
};

/// Support declared by mods std1 and tests1
const NAVAL_BOMBARD: &str = "Naval bombard";

#[cfg(feature = "test")]
static SUCCESS: AtomicBool = AtomicBool::new(false);
/// Center of a BrickWall tile (tile 3,15 of meadow1)
const WALL_POSITION: WorldVec2 = WorldVec2::new(17.5, 77.5);

const MAP: &str = "maps/meadow1";

/// Precision of smoke durations check (server fading tick, network and frames)
#[cfg(feature = "test")]
const SMOKE_TOLERANCE: Duration = Duration::from_millis(900);
/// Network and frames precision, whatever the speed
#[cfg(feature = "test")]
const SMOKE_MIN_TOLERANCE: Duration = Duration::from_millis(300);
/// Minimal test duration before considering it failed
#[cfg(feature = "test")]
const MIN_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: Case,

    #[arg(long)]
    test: bool,

    #[arg(long = "mod", default_value = "tests1")]
    mod_: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Case {
    Support,
    EndTime,
    EndMorale,
    ExplosionAlterWall,
    SmokeGrenade,
}

#[cfg(feature = "test")]
#[derive(Resource)]
struct Timeout(Duration);

#[derive(Resource)]
struct Center(WorldVec2);

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_from(crate::args());
    logging::setup_logging()?;

    #[cfg(not(feature = "test"))]
    if args.test {
        panic!("Execute with --test need `test` feature enabled")
    }

    let mod_path = PathBuf::from("mods").join(&args.mod_);
    let mod_ = Mod::load(&mod_path, None)?;
    let meta = Meta::from_file(&PathBuf::from(MAP).join("meta.toml"))?;
    let map = oc_world::reader::MapReader::new(&PathBuf::from(MAP));
    let map = map.context(format!("Read map {MAP}"))?;
    let mut w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    if matches!(args.case, Case::EndTime) {
        // Fight duration is in game time (server compares it to real elapsed * speed). Ensure at
        // least 1s of real time, so individual has time to move before the end even with a high
        // speed (OC_SPEED) on a loaded machine (e2e tests run in parallel)
        let real_min = Duration::from_secs(1).mul_f32(w.speed());
        w = w.with_fight_duration(Duration::from_secs(2).max(real_min));
    }
    let speed = w.speed();
    let center = WorldVec2::new(
        w.world_width_pixels() as f32 / 2.,
        w.world_height_pixels() as f32 / 2.,
    );

    let (phase, supports, individuals, squads) = match args.case {
        Case::EndTime => (
            BattlePhase::Deployment,
            Supports::default(),
            vec![
                TestIndividual::builder()
                    .side(Side::A)
                    .position(WorldVec3::new(center.x, center.y, 0.))
                    .build()
                    .make(&w),
            ],
            vec![
                TestSquad::builder()
                    .position(center)
                    .members(vec![oc_individual::IndividualIndex(0)])
                    .orders(vec![Order::MoveFastTo(WorldVec2::new(
                        center.x + 200.,
                        center.y,
                    ))])
                    .build()
                    .make(),
            ],
        ),
        Case::EndMorale => (
            BattlePhase::Deployment,
            Supports::default(),
            vec![
                TestIndividual::builder()
                    .side(Side::A)
                    .position(WorldVec3::new(center.x, center.y + 100., 0.))
                    .weapons(
                        TestWeapons::builder()
                            .primary(TestWeapon::filled(&mod_, "Weapon3").make())
                            .build()
                            .make(),
                    )
                    .build()
                    .make(&w),
                TestIndividual::builder()
                    .side(Side::B)
                    .position(WorldVec3::new(center.x, center.y, 0.))
                    .build()
                    .make(&w),
            ],
            vec![
                TestSquad::builder()
                    .position(WorldVec2::new(center.x, center.y + 100.))
                    .members(vec![oc_individual::IndividualIndex(0)])
                    .build()
                    .make(),
                TestSquad::builder()
                    .position(center)
                    .members(vec![oc_individual::IndividualIndex(1)])
                    .build()
                    .make(),
            ],
        ),
        Case::SmokeGrenade => {
            let smoke = mod_.grenade_from_name("Smoke grenade")?.index();
            let target = WorldVec2::new(center.x + Meters(10.).pixels(&w), center.y);
            (
                BattlePhase::Deployment,
                Supports::default(),
                vec![
                    TestIndividual::builder()
                        .side(Side::A)
                        .position(WorldVec3::new(center.x, center.y, 0.))
                        .weapons(TestWeapons::builder().grenades(vec![smoke]).build().make())
                        .build()
                        .make(&w),
                ],
                vec![
                    TestSquad::builder()
                        .position(center)
                        .members(vec![oc_individual::IndividualIndex(0)])
                        .orders(vec![Order::Smoke(target)])
                        .build()
                        .make(),
                ],
            )
        }
        Case::ExplosionAlterWall => (BattlePhase::Deployment, Supports::default(), vec![], vec![]),
        Case::Support => (
            BattlePhase::Deployment,
            Supports::new(
                Support(vec![(SupportType::new(NAVAL_BOMBARD), 1)]),
                Support::default(),
            ),
            vec![],
            vec![],
        ),
    };

    let snapshot = SnapshotBuilder::new(map, individuals, squads, vec![], phase, vec![], vec![])
        .with_supports(supports)
        .build(w, &mod_)?;

    let example = run::Example::builder()
        .world(PathBuf::from(MAP))
        .mod_(mod_path)
        .install(Box::new(move |app| setup(app, center, speed)))
        .snapshot(snapshot);
    example.build().run()?;

    #[cfg(feature = "test")]
    match SUCCESS.load(Ordering::Relaxed) {
        true => println!("✅ SUCCESS !"),
        false => return Err("❌ FAILED !".into()),
    }

    Ok(())
}

fn setup(app: &mut App, center: WorldVec2, #[allow(unused)] speed: f32) {
    let args = Args::parse_from(crate::args());

    app.insert_resource(Center(center));

    match args.case {
        Case::Support => {
            app.init_resource::<ConfigReceived>()
                .add_observer(on_game_config_received)
                .add_systems(Update, request_support_and_fight);
        }
        Case::ExplosionAlterWall => {
            #[cfg(feature = "debug")]
            app.add_observer(explosion_alter_wall);
            #[cfg(feature = "test")]
            app.add_systems(Update, alter_wall_test);
        }
        Case::SmokeGrenade => {
            app.add_observer(request_fight);
            #[cfg(feature = "test")]
            app.init_resource::<SmokeWatch>()
                .add_observer(on_smoke_update)
                .add_systems(Update, smoke_test);
        }
        Case::EndMorale => {
            app.add_observer(request_fight);
            #[cfg(feature = "test")]
            app.add_observer(on_battle_phase_morale);
        }
        Case::EndTime => {
            app.add_observer(request_fight);
            #[cfg(feature = "test")]
            app.init_resource::<EndWatch>()
                .add_observer(on_battle_phase)
                .add_systems(Update, watch_individual_after_end);
        }
    }

    #[cfg(feature = "test")]
    {
        // Success is detected as soon as it happens: a large minimal timeout does not slow down
        // tests, and let time to (maybe slow) app start and rendering
        let timeout = Duration::from_secs(30).div_f32(speed);
        app.insert_resource(Timeout(timeout.max(MIN_TIMEOUT)))
            .add_observer(on_fx)
            .add_systems(Update, timeout_exit);
    }
}

/// Fire an explosion near the BrickWall square
#[cfg(feature = "debug")]
fn explosion_alter_wall(_: On<GameConfigReceived>, mut commands: Commands) {
    let mod_ = Mod::load(
        &PathBuf::from("mods").join(Args::parse_from(crate::args()).mod_),
        None,
    )
    .unwrap();
    let shell = mod_
        .shell_from_name("Destruction Only (10 meters)")
        .unwrap()
        .index();

    commands.trigger(CenterCameraOn(WALL_POSITION));
    let explosion = ToServer::DebugExplosion(shell, WALL_POSITION);
    commands.trigger(ToServerEvent(explosion));
}

/// Wall tile must be altered into BrickWallDebris and its MixedDebris sprite displayed
#[cfg(feature = "test")]
fn alter_wall_test(
    g: Res<oc_battle_gui::states::GameConfig>,
    world: Res<oc_battle_gui::world::World>,
    sprites: Query<(&Sprite, &Transform), With<oc_battle_gui::ingame::tile::AlteredTileSprite>>,
    mut commands: Commands,
) {
    use oc_geo::tile::WorldTileIndex;
    use oc_root::{WcfgInto, WorldConfig};

    let_some!(g = &g.0, return);
    let w: &WorldConfig = &g.w;
    let tile: WorldTileIndex = WALL_POSITION.into_(w);
    let_some!(
        brick_debris = g.mod_.nature_index("BrickWallDebris"),
        return
    );
    let altered = world
        .altered_tiles
        .values()
        .filter_map(|tiles| tiles.get(&tile))
        .any(|t| t.nature == brick_debris);
    true_!(altered, return);

    // Only some sprites are displayed (minimum distance between them)
    let mixed_debris = 1; // See sprites/altered.png
    let positions: Vec<Vec2> = sprites
        .iter()
        .filter(|(sprite, _)| sprite.texture_atlas.as_ref().map(|a| a.index) == Some(mixed_debris))
        .map(|(_, transform)| transform.translation.truncate())
        .collect();
    true_!(!positions.is_empty(), return);
    let spaced = positions
        .iter()
        .enumerate()
        .all(|(i, a)| positions[i + 1..].iter().all(|b| a.distance(*b) >= 20.));
    if !spaced {
        eprintln!("❌ Debris sprites closer than 20px");
        commands.write_message(AppExit::from_code(0));
        return;
    }

    SUCCESS.store(true, Ordering::Relaxed);
    commands.write_message(AppExit::from_code(0));
}

// WARNING: AI generated
#[cfg(feature = "test")]
#[derive(Debug, Resource, Default)]
struct SmokeWatch {
    /// When the target tile smoke was first received
    seen: Option<std::time::Instant>,
    /// Same than `seen`, but in app time: smoke fx lifetime is ticked with it, and app time is
    /// slower than real one when frames are very slow (delta is clamped)
    seen_app: Duration,
    /// Opacity received at first
    initial: oc_utils::percent::Percent,
    /// Opacity changes received for target tile after the first one: when, and opacity
    changes: Vec<(std::time::Instant, oc_utils::percent::Percent)>,
}

// WARNING: AI generated
/// Follow target tile smoke from received updates (and not from world state at each frame): at
/// fast speed, smoke can appear and disappear between two (slow) frames.
#[cfg(feature = "test")]
fn on_smoke_update(
    update: On<oc_battle_gui::world::UpdateSmoke>,
    g: Res<oc_battle_gui::states::GameConfig>,
    center: Res<Center>,
    time: Res<Time>,
    mut watch: ResMut<SmokeWatch>,
) {
    let_some!(g = &g.0, return);
    let target = WorldVec2::new(center.0.x + Meters(10.).pixels(&g.w), center.0.y);
    let tile = oc_geo::tile::TileXy::from_([target.x, target.y], &g.w);
    let tile = oc_geo::tile::WorldTileIndex::from_(tile, &g.w);
    let Some((_, opacity)) = update.1.iter().find(|(i, _)| *i == tile) else {
        return;
    };
    let now = std::time::Instant::now();

    match watch.seen {
        None if !opacity.is_zero() => {
            watch.seen = Some(now);
            watch.seen_app = time.elapsed();
            watch.initial = *opacity;
        }
        None => {}
        Some(_) => watch.changes.push((now, *opacity)),
    }
}

// WARNING: AI generated
/// Smoke cloud must keep its full opacity (displayed and in world) during smoke_duration of
/// grenade, then be gone after smoke_fade_duration.
#[cfg(feature = "test")]
fn smoke_test(
    smokes: Query<(), With<oc_battle_gui::fx::Smoke>>,
    world: Res<oc_battle_gui::world::World>,
    g: Res<oc_battle_gui::states::GameConfig>,
    center: Res<Center>,
    time: Res<Time>,
    watch: Res<SmokeWatch>,
    mut commands: Commands,
) {
    let_some!(g = &g.0, return);
    let grenade = g.mod_.grenade_from_name("Smoke grenade").unwrap();
    // Durations are in game time, while elapsed time is real time
    let speed = g.w.speed();
    let hold = Duration::from_secs_f32(grenade.smoke_duration / speed);
    let fade = Duration::from_secs_f32(grenade.smoke_fade_duration / speed);
    // Frames can be slow (software rendering, parallel tests): updates are received at frame time
    let tolerance = SMOKE_TOLERANCE.div_f32(speed).max(SMOKE_MIN_TOLERANCE) + time.delta() * 2;

    let target = WorldVec2::new(center.0.x + Meters(10.).pixels(&g.w), center.0.y);
    let tile = oc_geo::tile::TileXy::from_([target.x, target.y], &g.w);
    let opacity = world.smoke_at(tile, &g.w);
    let displayed = !smokes.is_empty();

    let fail = |commands: &mut Commands, message: String| {
        eprintln!("❌ {message}");
        commands.write_message(AppExit::from_code(0));
    };

    let_some!(seen = watch.seen, return);
    // Opacity comes from server (real time), display from smoke fx (app time)
    let age = seen.elapsed();
    let app_age = time.elapsed().saturating_sub(watch.seen_app);

    // Opacity must not change during hold
    if let Some((at, changed)) = watch
        .changes
        .iter()
        .find(|(at, _)| at.duration_since(seen) + tolerance < hold)
    {
        let message = format!(
            "Smoke must be kept during {hold:?}, but at {:?}: opacity={changed:?} (initial {:?})",
            at.duration_since(seen),
            watch.initial
        );
        return fail(&mut commands, message);
    }

    // Smoke fx is spawned from a message which can be processed in the frame after the update one
    if app_age > time.delta() * 2 && app_age + tolerance < hold && !displayed {
        let message = format!("Smoke must be displayed during {hold:?}, but not at {app_age:?}");
        return fail(&mut commands, message);
    }

    // App time is not faster than real time: `age` is also passed
    if app_age > hold + fade + tolerance {
        if displayed || !opacity.is_zero() {
            let message = format!(
                "Smoke must be gone after {:?}, but at {age:?} ({app_age:?} app time): \
                 displayed={displayed}, opacity={opacity:?}",
                hold + fade
            );
            return fail(&mut commands, message);
        }
        SUCCESS.store(true, Ordering::Relaxed);
        commands.write_message(AppExit::from_code(0));
    }
}

#[derive(Resource, Default)]
struct ConfigReceived(bool);

fn on_game_config_received(_: On<GameConfigReceived>, mut received: ResMut<ConfigReceived>) {
    received.0 = true;
}

fn request_support_and_fight(
    mut received: ResMut<ConfigReceived>,
    center: Res<Center>,
    mut commands: Commands,
) {
    if !std::mem::take(&mut received.0) {
        return;
    }
    commands.trigger(ToServerEvent(ToServer::RequestSupport(
        SupportType::new(NAVAL_BOMBARD),
        center.0,
    )));
    commands.trigger(ToServerEvent(ToServer::Fight));
}

fn request_fight(_: On<GameConfigReceived>, mut commands: Commands) {
    commands.trigger(ToServerEvent(ToServer::Fight));
}

#[cfg(feature = "test")]
#[derive(Resource, Default)]
struct EndWatch {
    /// Frames elapsed since BattlePhase::End is received
    ended_since_frames: Option<u32>,
    // Position (and time) a few frames after BattlePhase::End is received
    end: Option<(Duration, WorldVec3)>,
}

/// Frames to wait after End before reading individual position: updates received with End (in the
/// same frame) are applied on components by deferred commands
#[cfg(feature = "test")]
const END_POSITION_FRAMES: u32 = 2;

#[cfg(feature = "test")]
fn individual_position(
    individuals: &Query<(
        &Position,
        &oc_battle_gui::entity::individual::IndividualIndex,
    )>,
) -> Option<WorldVec3> {
    individuals
        .iter()
        .find(|(_, i)| i.0 == oc_individual::IndividualIndex(0))
        .map(|(p, _)| p.0)
}

#[cfg(feature = "test")]
fn on_battle_phase(phase: On<SetBattlePhase>, mut watch: ResMut<EndWatch>, mut commands: Commands) {
    match (phase.phase(), phase.end()) {
        (oc_battle_gui::ingame::BattlePhase::End, Some(End::Time(_))) => {
            watch.ended_since_frames.get_or_insert(0);
        }
        (oc_battle_gui::ingame::BattlePhase::End, end) => {
            eprintln!("❌ Expected End::Time, got {:?}", end);
            commands.write_message(AppExit::from_code(0));
        }
        // Not ended yet (ex: start of fight)
        _ => {}
    }
}

#[cfg(feature = "test")]
fn on_battle_phase_morale(phase: On<SetBattlePhase>, mut commands: Commands) {
    true_!(
        phase.phase() == oc_battle_gui::ingame::BattlePhase::End,
        return
    );

    match phase.end() {
        Some(End::Morale(
            _,
            BattleResult {
                a: SideResult::TotalVictory,
                b: SideResult::TotalDefeat,
            },
        )) => SUCCESS.store(true, Ordering::Relaxed),
        other => eprintln!("❌ Unexpected end: {other:?}"),
    }
    commands.write_message(AppExit::from_code(0));
}

#[cfg(feature = "test")]
fn watch_individual_after_end(
    time: Res<Time>,
    individuals: Query<(
        &Position,
        &oc_battle_gui::entity::individual::IndividualIndex,
    )>,
    mut watch: ResMut<EndWatch>,
    g: Res<oc_battle_gui::states::GameConfig>,
    center: Res<Center>,
    mut commands: Commands,
) {
    let_some!(g = &g.0, return);
    let_some!(position = individual_position(&individuals), return);
    let_some!(ended_since_frames = watch.ended_since_frames, return);
    if watch.end.is_none() {
        if ended_since_frames < END_POSITION_FRAMES {
            watch.ended_since_frames = Some(ended_since_frames + 1);
            return;
        }
        watch.end = Some((time.elapsed(), position));
    }
    let_some!((end_time, end_position) = watch.end, return);

    if !position.almost_equal(end_position, 0.001) {
        eprintln!("❌ Individual moved after End ({end_position:?} -> {position:?})");
        commands.write_message(AppExit::from_code(0));
        return;
    }

    if time.elapsed()
        >= end_time
            + Duration::from_secs(2)
                .div_f32(g.w.speed())
                .max(Duration::from_millis(500))
    {
        // Compare with spawn position (individual is spawned at center): with high speed, individual
        // can already have moved when first seen by gui
        if WorldVec2::new(end_position.x, end_position.y).almost_equal(center.0, 0.001) {
            eprintln!("❌ Individual never moved before End");
        } else {
            SUCCESS.store(true, Ordering::Relaxed);
        }
        commands.write_message(AppExit::from_code(0));
    }
}

#[cfg(feature = "test")]
fn on_fx(fx: On<FxEvent>, center: Res<Center>, mut commands: Commands) {
    use oc_physics::fx::{Fx, Visual};

    let Fx::Visual(Visual::Explosion(position, _)) = &fx.0 else {
        return;
    };

    if position.almost_equal(center.0, 10.) {
        SUCCESS.store(true, Ordering::Relaxed);
        commands.write_message(AppExit::from_code(0));
    }
}

#[cfg(feature = "test")]
fn timeout_exit(time: Res<Time>, timeout: Res<Timeout>, mut commands: Commands) {
    if time.elapsed() > timeout.0 {
        eprintln!("❌ Timeout reached ({:?})", timeout.0);
        commands.write_message(AppExit::from_code(0));
    }
}
