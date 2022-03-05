use ggez::{GameError, GameResult};

use crate::{NetWorkMode, Opt};

#[derive(Debug, Clone)]
pub struct Config {
    network_mode: NetWorkMode,
    server_address: String,
    target_fps: u32,
    entity_update_freq: u64,
    entity_animate_freq: u64,
}

impl Config {
    pub fn new(opt: &Opt) -> GameResult<Self> {
        let (network_mode, server_address) = (opt.network_mode.clone(), opt.server_address.clone());

        Ok(Self {
            /// Modify engine behavior as server or client
            network_mode,
            /// Server address to bind, or connect to
            server_address,
            /// Target FPS of engine
            target_fps: 60,
            /// Frequency of entity update :
            ///  - World pixel position according to movement
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
}

// Width of sprite sheet
pub const SCENE_ITEMS_SPRITE_SHEET_WIDTH: f32 = 800.0;
// Height of sprite sheet
pub const SCENE_ITEMS_SPRITE_SHEET_HEIGHT: f32 = 600.0;
