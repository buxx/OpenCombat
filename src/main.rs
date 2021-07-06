use std::env;
use std::path;

use ggez::{event, GameResult};
use glam::Vec2;
use log;

use crate::physics::GridPoint;
use crate::scene::item::SceneItemModifier;
use crate::scene::main::MainStateModifier;
use scene::main::MainState;

mod audio;
mod behavior;
mod config;
mod gameplay;
mod map;
mod physics;
mod scene;
mod ui;
mod util;

/// Represent coordinates on the window
type WindowPoint = Vec2;
/// Represent an offset (used by example for moving on the scene)
type Offset = Vec2;
/// Represent coordinates on the battle scene
type ScenePoint = Vec2;
/// Represent scene item position in MainState.scene_items vector. It is used like an identifier
type SceneItemId = usize;
/// Represent a path in the map grid
type GridPath = Vec<GridPoint>;
/// Represent a frame number
type FrameI = u32;
/// Float representing meters
type Meters = f32;
/// Float representing a multiplier
type Factor = f32;
/// Squad id (position in main scene squad lists
type SquadId = usize;
/// Angle
type Angle = f32;

/// Main message enum, mostly used as function returns
pub enum Message {
    /// Message to apply on scene item to mutate it
    SceneItemMessage(SceneItemId, SceneItemModifier),
    /// Message to apply on MainState to mutate it
    MainStateMessage(MainStateModifier),
}

pub fn main() -> GameResult {
    env_logger::init();
    log::info!("Starting");
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    log::info!("Initializing context");
    let cb = ggez::ContextBuilder::new("oc", "bux")
        .add_resource_path(resource_dir)
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0));
    let (mut ctx, event_loop) = cb.build()?;

    log::info!("Loading state");
    let state = MainState::new(&mut ctx)?;
    log::info!("Display scene");
    event::run(ctx, event_loop, state)
}
