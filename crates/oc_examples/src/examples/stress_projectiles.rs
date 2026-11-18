#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::{f32::consts::TAU, time::Duration};

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use bevy::prelude::*;
use oc_battle_gui::ingame::GameConfigReceived;
use oc_battle_gui::{ingame::camera::move_::CenterCameraOn, network::output::ToServerEvent};
use oc_mod::{
    Mod, ammunition::IndexedAmmunition, armament::IndexedShotMode, weapons::IndexedWeapon,
};
use oc_network::ToServer;
use oc_projectile::spawn::SpawnProjectiles;
use oc_root::battle::BattlePhase;
use oc_root::geo::WorldVec2;
use oc_root::{WorldConfig, geo::WorldVec3, physics::Meters, side::Side};
use oc_world::meta::Meta;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let mod_ = PathBuf::from("mods/tests1");
    let mod__ = Mod::load(&mod_, None)?;
    let meta = Meta::from_file(&PathBuf::from("maps/world1/meta.toml"))?;
    let map_ = PathBuf::from("maps/world1");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    let snapshot = SnapshotBuilder::new(
        map,
        vec![],
        vec![],
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .build(w, &mod__)?;

    let example = run::Example::builder()
        .world(PathBuf::from("maps/world1"))
        .mod_(mod_)
        .install(Box::new(install))
        .snapshot(snapshot);
    example.build().run()?;

    Ok(())
}

#[derive(Resource)]
struct Config {
    mod_: Mod,
    orbit_radius: f32,
    center_x: f32,
    center_y: f32,
}

#[derive(Debug, Component)]
struct Orbiter {
    center: Vec3,
    angle: f32,
    radius: f32,
    speed: f32, // radians per second
}

fn install(app: &mut bevy::app::App) {
    // TODO: avoid repetition with main()
    let meta = Meta::from_file(&PathBuf::from("maps/world1/meta.toml")).unwrap();
    let map_ = PathBuf::from("maps/world1");
    let map = oc_world::reader::MapReader::new(&map_).unwrap();
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None).unwrap();
    // Move at the center of the world
    let center_x = (map.width().unwrap() as f32 * w.geo_pixels_per_tile() as f32) / 2.;
    let center_y = (map.height().unwrap() as f32 * w.geo_pixels_per_tile() as f32) / 2.;

    let config = Config {
        mod_,
        orbit_radius: 150.,
        center_x,
        center_y,
    };

    app.insert_resource(config)
        .add_systems(Startup, setup)
        .add_systems(Update, orbit)
        .add_observer(
            |_: On<GameConfigReceived>, config: Res<Config>, mut commands: Commands| {
                commands.trigger(CenterCameraOn(WorldVec2::new(
                    config.center_x,
                    config.center_y,
                )));
            },
        );
}

fn setup(mut commands: Commands, config: Res<Config>) {
    commands.spawn((
        Transform::from_xyz(config.center_x + config.orbit_radius, 0., 3.),
        Orbiter {
            center: Vec3::new(config.center_x, config.center_y, 15.0),
            angle: 0.0,
            radius: config.orbit_radius,
            speed: 5.0,
        },
    ));
    commands.spawn((
        Transform::from_xyz(config.center_x + config.orbit_radius, 0., 3.),
        Orbiter {
            center: Vec3::new(config.center_x, config.center_y, 15.0),
            angle: (0.0 + 5.0 * 1.0) % TAU,
            radius: config.orbit_radius,
            speed: 5.0,
        },
    ));
    commands.spawn((
        Transform::from_xyz(config.center_x + config.orbit_radius, 0., 3.),
        Orbiter {
            center: Vec3::new(config.center_x, config.center_y, 15.0),
            angle: (0.0 + 5.0 * 2.0) % TAU,
            radius: config.orbit_radius,
            speed: 5.0,
        },
    ));
}

fn orbit(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Orbiter, &mut Transform)>,
    config: Res<Config>,
) {
    for (mut orbiter, mut transform) in &mut query {
        orbiter.angle = (orbiter.angle + orbiter.speed * time.delta_secs()) % TAU;
        let x = orbiter.radius * orbiter.angle.cos();
        let y = orbiter.radius * orbiter.angle.sin();
        transform.translation = Vec3::new(x, y, 3.);

        let (weapon, ammunition, shot) = weapons(&config);
        let directions: Vec<WorldVec3> = vec![
            (WorldVec3::new(orbiter.center.x + x, orbiter.center.y + y, 500.)
                - WorldVec3::new(orbiter.center.x, orbiter.center.y, 500.))
            .normalize_or_zero(),
        ];
        commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(
            SpawnProjectiles::new(
                weapon.index(),
                ammunition.index(),
                shot.index(),
                1,
                [orbiter.center.x, orbiter.center.y, 500.].into(),
                directions,
                Side::A,
                Duration::ZERO,
                None,
            ),
        )));
    }
}

fn weapons<'a>(
    config: &'a Res<'a, Config>,
) -> (
    &'a IndexedWeapon,
    &'a IndexedAmmunition,
    &'a IndexedShotMode,
) {
    let weapon = config
        .mod_
        .weapons
        .iter()
        .find(|w| w.name() == "Weapon1")
        .unwrap();
    let ammunition = weapon
        .ammunitions()
        .iter()
        .find(|a| a.name() == "Ammo1")
        .unwrap();
    let shot = weapon
        .shots()
        .iter()
        .find(|s| s.name() == "Single")
        .unwrap();
    (weapon, ammunition, shot)
}
