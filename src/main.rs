use std::env;
use std::path;

use ggez::{event, GameResult};
use glam::Vec2;

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

type WindowPoint = Vec2;
type Offset = Vec2;
type ScenePoint = Vec2;
type SceneItemId = usize;
type GridPath = Vec<GridPoint>;
type FrameI = u32;

pub enum Message {
    SceneItemMessage(SceneItemId, SceneItemModifier),
    MainStateMessage(MainStateModifier),
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("oc", "bux")
        .add_resource_path(resource_dir)
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0));
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
