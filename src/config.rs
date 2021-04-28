// execute update code 60x per seconds
pub const TARGET_FPS: u32 = 60;
// execute meta code each 20 frames
pub const META_EACH: u32 = 20;
// execute physics code each 10 frames
pub const PHYSICS_EACH: u32 = 10;
// execute animate code each 30 frames
pub const ANIMATE_EACH: u32 = 60;
// change sprite animation tile 30 frames
pub const SPRITE_EACH: u32 = 10;
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
// Width of one grid tile
pub const GRID_TILE_WIDTH: f32 = 5.0;
// Height of one grid tile
pub const GRID_TILE_HEIGHT: f32 = 5.0;
//
pub const DEFAULT_SELECTED_SQUARE_SIDE: f32 = 14.0;
//
pub const DEFAULT_SELECTED_SQUARE_SIDE_HALF: f32 = DEFAULT_SELECTED_SQUARE_SIDE / 2.0;
//
pub const SCENE_ITEMS_CHANGE_ERR_MSG: &str = "scene_items content change !";
//
pub const DEBUG: bool = true;
