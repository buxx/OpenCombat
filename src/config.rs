use ggez::GameResult;

use crate::{NetworkMode, Opt};

///
pub const TARGET_FPS: u64 = 60;
pub const SOLDIER_UPDATE_FREQ: u64 = 1;
pub const SOLDIER_ANIMATE_FREQ: u64 = 20;
pub const INTERIORS_UPDATE_FREQ: u64 = 60;
pub const VISIBILITY_UPDATE_FREQ: u64 = 60;
pub const FEELING_DECREASING_FREQ: u64 = 60;
pub const PHYSICS_UPDATE_FREQ: u64 = 1;
///

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
// When compute visibility, configure here each pixels step of line which me considered
pub const VISIBILITY_PIXEL_STEPS: usize = 5;

#[derive(Debug, Clone)]
pub struct Config {
    pub network_mode: NetworkMode,
    pub server_rep_address: String,
    pub server_pub_address: String,
    pub target_fps: u64,
    pub soldier_update_freq: u64,
    pub soldier_animate_freq: u64,
    pub interiors_update_freq: u64,
    pub visibility_update_freq: u64,
    pub physics_update_freq: u64,
    pub feeling_decreasing_freq: u64,
}
impl Config {
    pub fn new(opt: &Opt) -> GameResult<Self> {
        let (network_mode, server_rep_address, server_pub_address) = (
            opt.network_mode.clone(),
            opt.server_rep_address.clone(),
            opt.server_pub_address.clone(),
        );

        Ok(Self {
            /// Modify engine behavior as server or client
            network_mode,
            ///
            server_rep_address,
            ///
            server_pub_address,
            /// Target FPS of engine
            target_fps: TARGET_FPS,
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
        })
    }

    pub fn target_fps(&self) -> u64 {
        self.target_fps
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

    pub fn network_mode(&self) -> &NetworkMode {
        &self.network_mode
    }

    pub fn server_rep_address(&self) -> String {
        self.server_rep_address.clone()
    }

    pub fn server_pub_address(&self) -> String {
        self.server_pub_address.clone()
    }
}
