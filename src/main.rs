use std::env;
use std::path;

use scene::main::MainState;

use ggez::{event, GameResult};
use glam::Vec2;

mod behavior;
mod physics;
mod scene;
mod ui;

// TODO: create a ScenePosition and a WindowPosition to be more explicit
type Point2 = Vec2;
type Vector2 = Vec2;

const TARGET_FPS: u32 = 60; // execute update code 60x per seconds
const META_EACH: u32 = 20; // execute meta code each 20 frames
const PHYSICS_EACH: u32 = 10; // execute physics code each 10 frames
const ANIMATE_EACH: u32 = 60; // execute animate code each 30 frames
const SPRITE_EACH: u32 = 10; // change sprite animation tile 30 frames
const MAX_FRAME_I: u32 = 4294967295; // max of frame_i used to calculate ticks
const DISPLAY_OFFSET_BY: f32 = 3.0; // pixel offset by tick when player move screen display
const DISPLAY_OFFSET_BY_SPEED: f32 = 10.0; // pixel offset by tick when player move screen display with speed
const SCENE_ITEMS_SPRITE_SHEET_WIDTH: f32 = 800.0; // Width of sprite sheet
const SCENE_ITEMS_SPRITE_SHEET_HEIGHT: f32 = 600.0; // Height of sprite sheet
const UI_SPRITE_SHEET_WIDTH: f32 = 800.0; // Width of sprite sheet
const UI_SPRITE_SHEET_HEIGHT: f32 = 600.0; // Height of sprite sheet
const GRID_TILE_WIDTH: f32 = 5.0; // Width of one grid tile
const GRID_TILE_HEIGHT: f32 = 5.0; // Height of one grid tile
const DEFAULT_SELECTED_SQUARE_SIDE: f32 = 14.0;
const DEFAULT_SELECTED_SQUARE_SIDE_HALF: f32 = DEFAULT_SELECTED_SQUARE_SIDE / 2.0;
const SCENE_ITEMS_CHANGE_ERR_MSG: &str = "scene_items content change !";

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
