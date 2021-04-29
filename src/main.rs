use std::env;
use std::path;

use ggez::{event, GameResult};
use glam::Vec2;

use scene::main::MainState;

mod behavior;
mod config;
mod physics;
mod scene;
mod ui;

type WindowPoint = Vec2;
type Offset = Vec2;
type ScenePoint = Vec2;
type Vector2 = Vec2;

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
