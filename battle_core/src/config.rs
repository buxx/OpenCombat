use crate::{behavior::Behavior, map::terrain::TileType};
use serde_derive::{Deserialize, Serialize};

pub const DEFAULT_SERVER_REP_ADDRESS: &str = "tcp://0.0.0.0:4255";
pub const DEFAULT_SERVER_PUB_ADDRESS: &str = "tcp://0.0.0.0:4256";
///
pub const TARGET_FPS: u64 = 60;
pub const SOLDIER_UPDATE_FREQ: u64 = 1;
pub const SOLDIER_ANIMATE_FREQ: u64 = 20;
pub const INTERIORS_UPDATE_FREQ: u64 = 60;
pub const VISIBILITY_UPDATE_FREQ: u64 = 60;
pub const FEELING_DECREASING_FREQ: u64 = 60;
pub const PHYSICS_UPDATE_FREQ: u64 = 1;
///
pub const VISIBILITY_IDLE_MODIFIER: f32 = 0.5;
pub const VISIBILITY_MOVE_TO_MODIFIER: f32 = 1.0;
pub const VISIBILITY_MOVE_FAST_TO_MODIFIER: f32 = 2.0;
pub const VISIBILITY_SNEAK_TO_MODIFIER: f32 = -0.3;
pub const VISIBILITY_DEFEND_MODIFIER: f32 = -0.3;
pub const VISIBILITY_HIDE_MODIFIER: f32 = -0.3;
pub const VISIBILITY_IN_VEHICLE_MODIFIER: f32 = 0.;
pub const VISIBILITY_SUPPRESS_FIRE_MODIFIER: f32 = 0.5;
pub const VISIBILITY_ENGAGE_MODIFIER: f32 = 0.5;
pub const VISIBILITY_DEAD_MODIFIER: f32 = 0.0;
pub const VISIBILITY_UNCONSCIOUS_MODIFIER: f32 = 0.0;
///
pub const TILE_TYPE_OPACITY_SHORT_GRASS: f32 = 0.0;
pub const TILE_TYPE_OPACITY_MIDDLE_GRASS: f32 = 0.025;
pub const TILE_TYPE_OPACITY_HIGH_GRASS: f32 = 0.1;
pub const TILE_TYPE_OPACITY_DIRT: f32 = 0.0;
pub const TILE_TYPE_OPACITY_CONCRETE: f32 = 0.0;
pub const TILE_TYPE_OPACITY_MUD: f32 = 0.02;
pub const TILE_TYPE_OPACITY_BRICK_WALL: f32 = 1.0;
//
pub const VISIBILITY_BY_LAST_FRAME_SHOOT: u64 = 180;
pub const VISIBILITY_BY_LAST_FRAME_SHOOT_DISTANCE: usize = 4;

// Width of sprite sheet
pub const UI_SPRITE_SHEET_WIDTH: f32 = 800.0;
// Height of sprite sheet
pub const UI_SPRITE_SHEET_HEIGHT: f32 = 600.0;
// Target FPS
// Velocity of move vector
pub const MOVE_VELOCITY: f32 = 5.0 / TARGET_FPS as f32;
// Velocity of move fast vector
pub const MOVE_FAST_VELOCITY: f32 = 10.0 / TARGET_FPS as f32;
// Velocity of move hide vector
pub const MOVE_HIDE_VELOCITY: f32 = 1.5 / TARGET_FPS as f32;
// Selection square size of selected soldier
pub const DEFAULT_SELECTED_SQUARE_SIDE: f32 = 14.0;
// Half selection square size of selected soldier
pub const DEFAULT_SELECTED_SQUARE_SIDE_HALF: f32 = DEFAULT_SELECTED_SQUARE_SIDE / 2.0;
// Selection square size of selectable zone (click)
pub const SOLDIER_SELECTABLE_SQUARE_SIDE: f32 = 14.0;
// Half selection square size of selectable zone (click)
pub const SOLDIER_SELECTABLE_SQUARE_SIDE_HALF: f32 = SOLDIER_SELECTABLE_SQUARE_SIDE / 2.0;
// Frames to wait to draw the pending order path finding
pub const PENDING_ORDER_PATH_FINDING_DRAW_FRAMES: u64 = (TARGET_FPS / 3) as u64;

pub const VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT: f32 = 100.;
pub const VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_COEFFICIENT: f32 = 100.;
pub const VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_DIFF: f32 = 50.;

// Grid distance to search cover point
pub const COVER_DISTANCE: i32 = 5;

// Visibility computing must consider firsts tiles differently
pub const VISIBILITY_FIRSTS: usize = 4;
pub const VISIBLE_STARTS_AT: f32 = 0.5;
// When compute visibility, configure here each pixels step of line which me considered
pub const VISIBILITY_PIXEL_STEPS: usize = 5;

pub trait TerrainTileOpacity {
    fn terrain_tile_opacity(&self, tile_type: &TileType) -> f32;
}
pub enum ConfigError {}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub send_debug_points: bool,
    pub soldier_update_freq: u64,
    pub soldier_animate_freq: u64,
    pub interiors_update_freq: u64,
    pub visibility_update_freq: u64,
    pub physics_update_freq: u64,
    pub feeling_decreasing_freq: u64,
    pub visibility_firsts: usize,
    pub visible_starts_at: f32,
    pub visibility_idle_modifier: f32,
    pub visibility_move_to_modifier: f32,
    pub visibility_move_fast_to_modifier: f32,
    pub visibility_sneak_to_modifier: f32,
    pub visibility_defend_modifier: f32,
    pub visibility_hide_modifier: f32,
    pub visibility_in_vehicle_modifier: f32,
    pub visibility_suppress_fire_modifier: f32,
    pub visibility_engage_modifier: f32,
    pub visibility_dead_modifier: f32,
    pub visibility_unconscious_modifier: f32,
    pub tile_type_opacity_short_grass: f32,
    pub tile_type_opacity_middle_grass: f32,
    pub tile_type_opacity_high_grass: f32,
    pub tile_type_opacity_dirt: f32,
    pub tile_type_opacity_concrete: f32,
    pub tile_type_opacity_mud: f32,
    pub tile_type_opacity_brick_wall: f32,
    pub visibility_by_last_frame_shoot: u64,
    pub visibility_by_last_frame_shoot_distance: usize,
}

impl ServerConfig {
    pub fn new() -> Self {
        Self {
            send_debug_points: false,
            /// Frequency of soldier update :
            ///  - World pixel point according to movement
            ///  - ...
            soldier_update_freq: SOLDIER_UPDATE_FREQ,
            /// Frequency of soldier animation :
            ///  - Compute visibility with other soldiers
            ///  - Compute behavior against physics (explosions, gunfires, ...)
            soldier_animate_freq: SOLDIER_ANIMATE_FREQ,
            /// Frequency of update buildings interiors visibility
            interiors_update_freq: INTERIORS_UPDATE_FREQ,
            /// Frequency of update visibility between soldiers
            visibility_update_freq: VISIBILITY_UPDATE_FREQ,
            ///
            physics_update_freq: PHYSICS_UPDATE_FREQ,
            /// Frequency of decreasing feelings
            feeling_decreasing_freq: FEELING_DECREASING_FREQ,
            ///
            visibility_firsts: VISIBILITY_FIRSTS,
            visible_starts_at: VISIBLE_STARTS_AT,

            visibility_idle_modifier: VISIBILITY_IDLE_MODIFIER,
            visibility_move_to_modifier: VISIBILITY_MOVE_TO_MODIFIER,
            visibility_move_fast_to_modifier: VISIBILITY_MOVE_FAST_TO_MODIFIER,
            visibility_sneak_to_modifier: VISIBILITY_SNEAK_TO_MODIFIER,
            visibility_defend_modifier: VISIBILITY_DEFEND_MODIFIER,
            visibility_hide_modifier: VISIBILITY_HIDE_MODIFIER,
            visibility_in_vehicle_modifier: VISIBILITY_IN_VEHICLE_MODIFIER,
            visibility_suppress_fire_modifier: VISIBILITY_SUPPRESS_FIRE_MODIFIER,
            visibility_engage_modifier: VISIBILITY_ENGAGE_MODIFIER,
            visibility_dead_modifier: VISIBILITY_DEAD_MODIFIER,
            visibility_unconscious_modifier: VISIBILITY_UNCONSCIOUS_MODIFIER,
            visibility_by_last_frame_shoot: VISIBILITY_BY_LAST_FRAME_SHOOT,
            visibility_by_last_frame_shoot_distance: VISIBILITY_BY_LAST_FRAME_SHOOT_DISTANCE,

            tile_type_opacity_short_grass: TILE_TYPE_OPACITY_SHORT_GRASS,
            tile_type_opacity_middle_grass: TILE_TYPE_OPACITY_MIDDLE_GRASS,
            tile_type_opacity_high_grass: TILE_TYPE_OPACITY_HIGH_GRASS,
            tile_type_opacity_dirt: TILE_TYPE_OPACITY_DIRT,
            tile_type_opacity_concrete: TILE_TYPE_OPACITY_CONCRETE,
            tile_type_opacity_mud: TILE_TYPE_OPACITY_MUD,
            tile_type_opacity_brick_wall: TILE_TYPE_OPACITY_BRICK_WALL,
        }
    }

    pub fn soldier_update_freq(&self) -> u64 {
        self.soldier_update_freq
    }

    pub fn soldier_animate_freq(&self) -> u64 {
        self.soldier_animate_freq
    }

    pub fn interiors_update_freq(&self) -> u64 {
        self.interiors_update_freq
    }

    pub fn visibility_update_freq(&self) -> u64 {
        self.visibility_update_freq
    }

    pub fn physics_update_freq(&self) -> u64 {
        self.physics_update_freq
    }

    pub fn feeling_decreasing_freq(&self) -> u64 {
        self.feeling_decreasing_freq
    }

    pub fn visibility_behavior_modifier(&self, behavior: &Behavior) -> f32 {
        match behavior {
            Behavior::Idle => self.visibility_idle_modifier,
            Behavior::Hide(_) => self.visibility_hide_modifier,
            Behavior::Defend(_) => self.visibility_defend_modifier,
            Behavior::MoveTo(_) => self.visibility_move_to_modifier,
            Behavior::MoveFastTo(_) => self.visibility_move_fast_to_modifier,
            Behavior::SneakTo(_) => self.visibility_sneak_to_modifier,
            Behavior::DriveTo(_) => self.visibility_in_vehicle_modifier,
            Behavior::RotateTo(_) => self.visibility_in_vehicle_modifier,
            Behavior::SuppressFire(_) => self.visibility_suppress_fire_modifier,
            Behavior::EngageSoldier(_) => self.visibility_engage_modifier,
            Behavior::Dead => self.visibility_dead_modifier,
            Behavior::Unconscious => self.visibility_unconscious_modifier,
        }
    }

    pub fn behavior_velocity(&self, behavior: &Behavior) -> Option<f32> {
        match behavior {
            Behavior::Idle => None,
            Behavior::MoveTo(_) => Some(MOVE_VELOCITY),
            Behavior::MoveFastTo(_) => Some(MOVE_FAST_VELOCITY),
            Behavior::SneakTo(_) => Some(MOVE_HIDE_VELOCITY),
            Behavior::Defend(_) => None,
            Behavior::Hide(_) => None,
            Behavior::DriveTo(_) => None,
            Behavior::RotateTo(_) => None,
            Behavior::Dead => None,
            Behavior::Unconscious => None,
            Behavior::SuppressFire(_) => None,
            Behavior::EngageSoldier(_) => None,
        }
    }

    pub fn react(&mut self, message: &ChangeConfigMessage) {
        match message {
            ChangeConfigMessage::SendDebugPoints(value) => self.send_debug_points = *value,
        }
    }
}

pub struct GuiConfig {
    pub target_fps: u32,
    pub interiors_update_freq: u64,
    pub tile_type_opacity_short_grass: f32,
    pub tile_type_opacity_middle_grass: f32,
    pub tile_type_opacity_high_grass: f32,
    pub tile_type_opacity_dirt: f32,
    pub tile_type_opacity_concrete: f32,
    pub tile_type_opacity_mud: f32,
    pub tile_type_opacity_brick_wall: f32,
}

impl GuiConfig {
    pub fn new() -> Self {
        Self {
            target_fps: TARGET_FPS as u32,
            interiors_update_freq: INTERIORS_UPDATE_FREQ,
            tile_type_opacity_short_grass: TILE_TYPE_OPACITY_SHORT_GRASS,
            tile_type_opacity_middle_grass: TILE_TYPE_OPACITY_MIDDLE_GRASS,
            tile_type_opacity_high_grass: TILE_TYPE_OPACITY_HIGH_GRASS,
            tile_type_opacity_dirt: TILE_TYPE_OPACITY_DIRT,
            tile_type_opacity_concrete: TILE_TYPE_OPACITY_CONCRETE,
            tile_type_opacity_mud: TILE_TYPE_OPACITY_MUD,
            tile_type_opacity_brick_wall: TILE_TYPE_OPACITY_BRICK_WALL,
        }
    }
}

impl TerrainTileOpacity for GuiConfig {
    fn terrain_tile_opacity(&self, tile_type: &TileType) -> f32 {
        match tile_type {
            TileType::ShortGrass => self.tile_type_opacity_short_grass,
            TileType::MiddleGrass => self.tile_type_opacity_middle_grass,
            TileType::HighGrass => self.tile_type_opacity_middle_grass,
            TileType::Dirt => self.tile_type_opacity_dirt,
            TileType::Concrete => self.tile_type_opacity_concrete,
            TileType::Mud => self.tile_type_opacity_mud,
            TileType::BrickWall => self.tile_type_opacity_brick_wall,
        }
    }
}

impl TerrainTileOpacity for ServerConfig {
    fn terrain_tile_opacity(&self, tile_type: &TileType) -> f32 {
        match tile_type {
            TileType::ShortGrass => self.tile_type_opacity_short_grass,
            TileType::MiddleGrass => self.tile_type_opacity_middle_grass,
            TileType::HighGrass => self.tile_type_opacity_middle_grass,
            TileType::Dirt => self.tile_type_opacity_dirt,
            TileType::Concrete => self.tile_type_opacity_concrete,
            TileType::Mud => self.tile_type_opacity_mud,
            TileType::BrickWall => self.tile_type_opacity_brick_wall,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChangeConfigMessage {
    SendDebugPoints(bool),
}
