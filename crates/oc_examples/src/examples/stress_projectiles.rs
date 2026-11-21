#![allow(clippy::unwrap_used)]

use std::f32::consts::TAU;
use std::path::PathBuf;

use crate::{
    logging, run,
    snapshot::SnapshotBuilder,
    utils::{Firing, Loaded, direction},
};
use bevy::prelude::*;
use oc_battle_gui::ingame::GameConfigReceived;
use oc_battle_gui::{ingame::camera::move_::CenterCameraOn, network::output::ToServerEvent};
use oc_network::ToServer;
use oc_root::geo::WorldVec2;
use oc_root::side::Side;

const MOD: &str = "mods/tests1";
const MAP: &str = "maps/world1";

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let loaded = Loaded::new(&PathBuf::from(MOD), &PathBuf::from(MAP))?;
    // Move at the center of the world
    let config = Config {
        firing: Firing::weapon1(&loaded.mod_),
        orbit_radius: 150.,
        center_x: loaded.w.world_width_pixels() as f32 / 2.,
        center_y: loaded.w.world_height_pixels() as f32 / 2.,
    };
    let snapshot =
        SnapshotBuilder::fight(loaded.map, vec![], vec![]).build(loaded.w, &loaded.mod_)?;

    let example = run::Example::builder()
        .world(PathBuf::from(MAP))
        .mod_(PathBuf::from(MOD))
        .install(Box::new(move |app| install(app, config.clone())))
        .snapshot(snapshot);
    example.build().run()?;

    Ok(())
}

#[derive(Resource, Clone)]
struct Config {
    firing: Firing,
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

fn install(app: &mut bevy::app::App, config: Config) {
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
    for i in 0..3 {
        commands.spawn((
            Transform::from_xyz(config.center_x + config.orbit_radius, 0., 3.),
            Orbiter {
                center: Vec3::new(config.center_x, config.center_y, 15.0),
                angle: (5.0 * i as f32) % TAU,
                radius: config.orbit_radius,
                speed: 5.0,
            },
        ));
    }
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

        let from = [orbiter.center.x, orbiter.center.y, 500.];
        let directions = vec![direction(
            from,
            [orbiter.center.x + x, orbiter.center.y + y, 500.],
        )];
        commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(
            config.firing.spawn(1, from, directions, Side::A),
        )));
    }
}
