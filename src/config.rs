use ggez::GameResult;

use crate::{NetWorkMode, Opt};

#[derive(Debug, Clone)]
pub struct Config {
    network_mode: NetWorkMode,
    server_rep_address: String,
    server_pub_address: String,
    target_fps: u32,
    entity_update_freq: u64,
    entity_animate_freq: u64,
}

// Width of sprite sheet
pub const SCENE_ITEMS_SPRITE_SHEET_WIDTH: f32 = 800.0;
// Height of sprite sheet
pub const SCENE_ITEMS_SPRITE_SHEET_HEIGHT: f32 = 600.0;
// Width of sprite sheet
pub const UI_SPRITE_SHEET_WIDTH: f32 = 800.0;
// Height of sprite sheet
pub const UI_SPRITE_SHEET_HEIGHT: f32 = 600.0;
// Target FPS
pub const TARGET_FPS: u32 = 60;
// Velocity of move vector
pub const MOVE_VELOCITY: f32 = 5.0 / TARGET_FPS as f32;
// Velocity of move fast vector
pub const MOVE_FAST_VELOCITY: f32 = 10.0 / TARGET_FPS as f32;
// Velocity of move hide vector
pub const MOVE_HIDE_VELOCITY: f32 = 1.5 / TARGET_FPS as f32;
// Selection square size of selected entity
pub const DEFAULT_SELECTED_SQUARE_SIDE: f32 = 14.0;
// Half selection square size of selected entity
pub const DEFAULT_SELECTED_SQUARE_SIDE_HALF: f32 = DEFAULT_SELECTED_SQUARE_SIDE / 2.0;
// Selection square size of selectable zone (click)
pub const SOLDIER_SELECTABLE_SQUARE_SIDE: f32 = 14.0;
// Half selection square size of selectable zone (click)
pub const SOLDIER_SELECTABLE_SQUARE_SIDE_HALF: f32 = SOLDIER_SELECTABLE_SQUARE_SIDE / 2.0;
// Frames to wait to draw the pending order path finding
pub const PENDING_ORDER_PATH_FINDING_DRAW_FRAMES: u64 = (TARGET_FPS / 3) as u64;

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
            /// Frequency of entity update :
            ///  - World pixel point according to movement
            ///  - ...
            entity_update_freq: 1,
            /// Frequency of entity animation :
            ///  - Compute visibility with other entities
            ///  - Compute behavior against physics (explosions, gunfires, ...)
            entity_animate_freq: 20,
        })
    }

    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }

    pub fn entity_update_freq(&self) -> u64 {
        self.entity_update_freq
    }

    pub fn entity_animate_freq(&self) -> u64 {
        self.entity_animate_freq
    }

    pub fn network_mode(&self) -> &NetWorkMode {
        &self.network_mode
    }

    pub fn server_rep_address(&self) -> String {
        self.server_rep_address.clone()
    }

    pub fn server_pub_address(&self) -> String {
        self.server_pub_address.clone()
    }
}
