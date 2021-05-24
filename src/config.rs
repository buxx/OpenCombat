// execute update code 60x per seconds
pub const TARGET_FPS: u32 = 60;
// execute physics code each 10 frames
pub const PHYSICS_EACH: u32 = 10;
// execute animate code each 15 frames
pub const ANIMATE_EACH: u32 = 15;
// execute seek code each 60 frames
pub const SEEK_EACH: u32 = 60;
// change sprite animation tile each 10 frames
pub const SPRITE_EACH: u32 = 10;
// compute interior sprites each 60 frames
pub const INTERIORS_EACH: u32 = 60;
// max of frame_i used to calculate ticks
pub const MAX_FRAME_I: u32 = 4294967295;
// pixel offset by tick when player move screen display
pub const DISPLAY_OFFSET_BY: f32 = 3.0;
// pixel offset by tick when player move screen display with speed
pub const DISPLAY_OFFSET_BY_SPEED: f32 = 10.0;
// Width of sprite sheet
pub const SCENE_ITEMS_SPRITE_SHEET_WIDTH: f32 = 800.0;
// Height of sprite sheet
pub const SCENE_ITEMS_SPRITE_SHEET_HEIGHT: f32 = 600.0;
// Width of sprite sheet
pub const UI_SPRITE_SHEET_WIDTH: f32 = 800.0;
// Height of sprite sheet
pub const UI_SPRITE_SHEET_HEIGHT: f32 = 600.0;
//
pub const DEFAULT_SELECTED_SQUARE_SIDE: f32 = 14.0;
//
pub const DEFAULT_SELECTED_SQUARE_SIDE_HALF: f32 = DEFAULT_SELECTED_SQUARE_SIDE / 2.0;
//
pub const SCENE_ITEMS_CHANGE_ERR_MSG: &str = "scene_items content change !";
// Distance from move target point to consider reached
pub const MOVE_TO_REACHED_WHEN_DISTANCE_INFERIOR_AT: f32 = 3.0;
// Velocity of move vector
pub const MOVE_VELOCITY: f32 = 1.0;
// Velocity of move fast vector
pub const MOVE_FAST_VELOCITY: f32 = 2.0;
// Velocity of move hide vector
pub const MOVE_HIDE_VELOCITY: f32 = 0.5;
// Visibility computing must ignore firsts tiles
pub const VISIBILITY_IGNORE_FIRSTS: usize = 4;
