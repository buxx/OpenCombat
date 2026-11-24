#![cfg_attr(not(feature = "test"), allow(dead_code, unused_imports))]
#![allow(clippy::unwrap_used)]

#[cfg(feature = "test")]
use oc_root::WcfgFrom;
use std::path::PathBuf;
#[cfg(feature = "test")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::{
    logging, run,
    snapshot::SnapshotBuilder,
    utils::{self, Loaded, NAVAL_BOMBARD, armed_soldier, soldier},
};
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
    battle::BattlePhase,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    side::Side,
};
#[cfg(feature = "test")]
use oc_utils::{d2::AlmostEqual, let_some, true_};
use oc_world::support::{SupportType, Supports};
use tests::{individual::TestIndividual, squad::TestSquad, weapons::TestWeapons};

#[cfg(feature = "test")]
static SUCCESS: AtomicBool = AtomicBool::new(false);
/// Center of a BrickWall tile (tile 3,15 of meadow1)
const WALL_POSITION: WorldVec2 = WorldVec2::new(17.5, 77.5);

const MAP: &str = "maps/meadow1";

/// WorldPersist: offsets (pixels, from map center) of persisted crater and wreck. Altered tile is
/// at map center.
const PERSIST_CRATER_OFFSET: f32 = 40.;
const PERSIST_WRECK_OFFSET: f32 = -40.;
const PERSIST_ALTERED_NATURE: &str = "BrickWallDebris";
const PERSIST_WRECK_VEHICLE: &str = "Tank1";
/// BulletCollideWreck: bullet is fired from this distance (pixels, west of the wreck), at this
/// height above ground (meters, under wreck chassis height)
const BULLET_FROM_WRECK: f32 = 60.;
const BULLET_HEIGHT: Meters = Meters(1.);
/// BulletCollideWreck: let time to the bullet to reach the wreck (game time)
#[cfg(feature = "test")]
const BULLET_DURATION: Duration = Duration::from_secs(3);

/// WreckLov: watcher (side A) is west of the wreck, hidden enemy (side B) is east of it (pixels,
/// from wreck center). Control enemy is not behind the wreck (wreck chassis is north oriented: its
/// length is along y axis)
const LOV_WATCHER_FROM_WRECK: WorldVec2 = WorldVec2::new(-50., 0.);
const LOV_HIDDEN_FROM_WRECK: WorldVec2 = WorldVec2::new(30., 0.);
const LOV_CONTROL_FROM_WRECK: WorldVec2 = WorldVec2::new(30., 60.);
/// WreckLov: duration (game time) during which control must be visible and hidden one not
#[cfg(feature = "test")]
const LOV_HOLD: Duration = Duration::from_secs(2);
/// WreckLov: minimal real duration of hold (network and frames precision)
#[cfg(feature = "test")]
const LOV_MIN_HOLD: Duration = Duration::from_secs(1);

/// Bullet (from, to) fired by BulletCollideWreck case
#[derive(Resource, Clone, Copy)]
struct Bullet(WorldVec3, WorldVec3);

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
    WorldPersist,
    BulletCollideWreck,
    WreckLov,
}

#[cfg(feature = "test")]
#[derive(Resource)]
struct Timeout(Duration);

#[derive(Resource)]
struct Center(WorldVec2);

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_from(crate::args());
    logging::setup_logging()?;

    utils::ensure_test_feature(args.test);

    let mod_path = PathBuf::from("mods").join(&args.mod_);
    let Loaded {
        mod_, map, mut w, ..
    } = Loaded::new(&mod_path, &PathBuf::from(MAP))?;
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

    let tiles = map.tiles(&w, &mod_)?;
    let wreck = utils::on_ground(&w, &tiles, offset_x(center, PERSIST_WRECK_OFFSET));
    let wreck_xy = WorldVec2::new(wreck.x, wreck.y);

    let (phase, supports, individuals, squads) = match args.case {
        Case::EndTime => (
            BattlePhase::Deployment,
            Supports::default(),
            vec![soldier(&w, Side::A, center.extend(0.))],
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
                armed_soldier(
                    &w,
                    &mod_,
                    Side::A,
                    [center.x, center.y + 100., 0.],
                    "Weapon3",
                ),
                soldier(&w, Side::B, center.extend(0.)),
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
        Case::WreckLov => {
            let at = |offset: WorldVec2| {
                utils::on_ground(
                    &w,
                    &tiles,
                    WorldVec2::new(wreck_xy.x + offset.x, wreck_xy.y + offset.y),
                )
            };
            let individuals = vec![
                soldier(&w, Side::A, at(LOV_WATCHER_FROM_WRECK)),
                soldier(&w, Side::B, at(LOV_HIDDEN_FROM_WRECK)),
                soldier(&w, Side::B, at(LOV_CONTROL_FROM_WRECK)),
            ];
            let squads = individuals
                .iter()
                .enumerate()
                .map(|(i, individual)| {
                    TestSquad::builder()
                        .position(WorldVec2::new(individual.position.x, individual.position.y))
                        .members(vec![oc_individual::IndividualIndex(i as u64)])
                        .build()
                        .make()
                })
                .collect();
            (BattlePhase::Fight, Supports::default(), individuals, squads)
        }
        Case::ExplosionAlterWall | Case::WorldPersist | Case::BulletCollideWreck => {
            (BattlePhase::Deployment, Supports::default(), vec![], vec![])
        }
        Case::Support => (
            BattlePhase::Deployment,
            utils::naval_bombards(1),
            vec![],
            vec![],
        ),
    };

    let bullet = matches!(args.case, Case::BulletCollideWreck).then(|| {
        let height = BULLET_HEIGHT.pixels(&w);
        let from = utils::on_ground(
            &w,
            &tiles,
            offset_x(center, PERSIST_WRECK_OFFSET - BULLET_FROM_WRECK),
        );
        Bullet(
            WorldVec3::new(from.x, from.y, from.z + height),
            WorldVec3::new(wreck.x, wreck.y, wreck.z + height),
        )
    });

    let mut builder = SnapshotBuilder::fight(map, individuals, squads)
        .with_phase(phase)
        .with_supports(supports);
    if matches!(
        args.case,
        Case::WorldPersist | Case::BulletCollideWreck | Case::WreckLov
    ) {
        builder = builder.with_persisted(persisted_map(&w, &mod_, center, wreck)?);
    }
    let snapshot = builder.build(w, &mod_)?;

    let example = run::Example::builder()
        .world(PathBuf::from(MAP))
        .mod_(mod_path)
        .install(Box::new(move |app| setup(app, center, speed, bullet)))
        .snapshot(snapshot);
    #[cfg(feature = "test")]
    let tracker = example.build().run()?;
    #[cfg(not(feature = "test"))]
    example.build().run()?;

    // Collisions are known by server (tracker), and not by gui
    #[cfg(feature = "test")]
    if matches!(args.case, Case::BulletCollideWreck) {
        let collisions = utils::tracked::wreck_collisions(&tracker.take());
        if collisions == 0 {
            eprintln!("❌ Bullet did not collide with wreck");
        }
        SUCCESS.store(collisions > 0, Ordering::Relaxed);
    }

    #[cfg(feature = "test")]
    utils::test_result(&SUCCESS)?;

    Ok(())
}

fn offset_x(position: WorldVec2, x: f32) -> WorldVec2 {
    WorldVec2::new(position.x + x, position.y)
}

/// Generate (on the fly) a persisted map (as saved at the end of a previous battle) containing an
/// altered tile, a crater and a wreck
fn persisted_map(
    w: &oc_root::WorldConfig,
    mod_: &Mod,
    center: WorldVec2,
    wreck: WorldVec3,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use oc_geo::tile::WorldTileIndex;
    use oc_root::{WcfgInto, craters::CraterType, format::RkyvFile, side::BySide};
    use oc_utils::d2::Direction;
    use oc_world::{crater::Crater, snapshot::State, tile::AlteredTile, wreck::Wreck};

    let count = w.tiles_count() as usize;

    let mut altered_tiles = vec![None; count];
    let tile: WorldTileIndex = center.into_(w);
    let nature = mod_
        .nature_index(PERSIST_ALTERED_NATURE)
        .ok_or(format!("Nature {PERSIST_ALTERED_NATURE} not found"))?;
    altered_tiles[tile.0 as usize] = Some(AlteredTile { i: tile, nature });

    let craters = vec![Crater::new(
        offset_x(center, PERSIST_CRATER_OFFSET),
        CraterType::Mud,
        0,
        30.,
    )];

    let wreck = Wreck::Vehicle {
        type_: mod_.vehicle_from_name(PERSIST_WRECK_VEHICLE)?.index(),
        position: wreck,
        chassis_direction: Direction::NORTH,
    };

    let state = State::new(
        altered_tiles,
        vec![None; count],
        craters,
        vec![wreck],
        BySide::new(vec![], vec![]),
        vec![],
        Default::default(),
    );
    let (_, path) = tempfile::NamedTempFile::new()?.keep()?;
    state.save(&path)?;

    Ok(path)
}

fn setup(app: &mut App, center: WorldVec2, #[allow(unused)] speed: f32, bullet: Option<Bullet>) {
    let args = Args::parse_from(crate::args());

    app.insert_resource(Center(center));
    if let Some(bullet) = bullet {
        app.insert_resource(bullet);
    }

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
        Case::WorldPersist => {
            app.add_observer(center_camera);
            #[cfg(feature = "test")]
            app.add_systems(Update, world_persist_test);
        }
        Case::WreckLov => {
            app.add_observer(center_camera_on_wreck);
            #[cfg(feature = "test")]
            app.init_resource::<LovWatch>()
                .add_systems(Update, wreck_lov_test);
        }
        Case::BulletCollideWreck => {
            app.add_observer(fire_bullet);
            #[cfg(feature = "test")]
            app.add_systems(Update, bullet_exit);
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

/// Fire one bullet towards the wreck
fn fire_bullet(_: On<GameConfigReceived>, bullet: Res<Bullet>, mut commands: Commands) {
    let mod_ = Mod::load(
        &PathBuf::from("mods").join(Args::parse_from(crate::args()).mod_),
        None,
    )
    .unwrap();
    let Bullet(from, to) = *bullet;

    commands.trigger(CenterCameraOn(WorldVec2::new(to.x, to.y)));
    let spawn =
        utils::Firing::weapon1(&mod_).spawn(1, from, vec![utils::direction(from, to)], Side::A);
    commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
}

/// Exit when bullet had time to reach the wreck (result is read from server tracker)
#[cfg(feature = "test")]
fn bullet_exit(
    game: Res<oc_battle_gui::states::Game>,
    g: Res<oc_battle_gui::states::GameConfig>,
    mut commands: Commands,
) {
    let_some!(g = &g.0, return);
    // Minimal duration to let time to app start and server exchanges
    let duration = BULLET_DURATION
        .div_f32(g.w.speed())
        .max(Duration::from_secs(1));
    if game.started.elapsed() > duration {
        commands.write_message(AppExit::from_code(0));
    }
}

fn center_camera_on_wreck(_: On<GameConfigReceived>, center: Res<Center>, mut commands: Commands) {
    commands.trigger(CenterCameraOn(offset_x(center.0, PERSIST_WRECK_OFFSET)));
}

/// Since when control enemy is visible while hidden one is not
#[cfg(feature = "test")]
#[derive(Resource, Default)]
struct LovWatch(Option<std::time::Instant>);

/// Enemy behind the wreck must stay not visible while control enemy (same distance, not behind the
/// wreck) is visible
#[cfg(feature = "test")]
fn wreck_lov_test(
    g: Res<oc_battle_gui::states::GameConfig>,
    individuals: Query<(
        &oc_battle_gui::entity::individual::IndividualIndex,
        &Visibility,
    )>,
    wrecks: Query<(), With<oc_battle_gui::ingame::wreck::WreckSprite>>,
    mut watch: ResMut<LovWatch>,
    mut commands: Commands,
) {
    let_some!(g = &g.0, return);
    true_!(!wrecks.is_empty(), return);
    let visible = |index: u64| {
        individuals.iter().find_map(|(i, v)| {
            (i.0 == oc_individual::IndividualIndex(index)).then_some(*v == Visibility::Visible)
        })
    };
    let_some!(hidden = visible(1), return);
    let_some!(control = visible(2), return);

    if !control {
        if watch.0.is_some() {
            eprintln!("❌ Control enemy became not visible");
            commands.write_message(AppExit::from_code(0));
        }
        return;
    }
    if hidden {
        eprintln!("❌ Enemy behind the wreck is visible");
        commands.write_message(AppExit::from_code(0));
        return;
    }

    let since = *watch.0.get_or_insert_with(std::time::Instant::now);
    if since.elapsed() >= LOV_HOLD.div_f32(g.w.speed()).max(LOV_MIN_HOLD) {
        SUCCESS.store(true, Ordering::Relaxed);
        commands.write_message(AppExit::from_code(0));
    }
}

fn center_camera(_: On<GameConfigReceived>, center: Res<Center>, mut commands: Commands) {
    commands.trigger(CenterCameraOn(center.0));
}

/// Persisted altered tile, crater and wreck must be in gui world (and displayed)
#[cfg(feature = "test")]
fn world_persist_test(
    g: Res<oc_battle_gui::states::GameConfig>,
    world: Res<oc_battle_gui::world::World>,
    center: Res<Center>,
    craters: Query<&Transform, With<oc_battle_gui::ingame::crater::CraterSprite>>,
    wrecks: Query<&Transform, With<oc_battle_gui::ingame::wreck::WreckSprite>>,
    mut commands: Commands,
) {
    use oc_geo::tile::WorldTileIndex;
    use oc_root::{WcfgFrom, WcfgInto, geo::ScreenVec2};

    let_some!(g = &g.0, return);
    let w = &g.w;
    let_some!(nature = g.mod_.nature_index(PERSIST_ALTERED_NATURE), return);

    let tile: WorldTileIndex = center.0.into_(w);
    let altered = world
        .altered_tiles
        .values()
        .filter_map(|tiles| tiles.get(&tile))
        .any(|t| t.nature == nature);
    true_!(altered, return);

    let near = |transform: &Transform, position: WorldVec2| {
        let screen = ScreenVec2::from_(position.extend(0.), w);
        transform
            .translation
            .truncate()
            .distance(Vec2::new(screen.x, screen.y))
            < 1.
    };
    let crater = offset_x(center.0, PERSIST_CRATER_OFFSET);
    true_!(craters.iter().any(|t| near(t, crater)), return);
    let wreck = offset_x(center.0, PERSIST_WRECK_OFFSET);
    true_!(wrecks.iter().any(|t| near(t, wreck)), return);

    SUCCESS.store(true, Ordering::Relaxed);
    commands.write_message(AppExit::from_code(0));
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
