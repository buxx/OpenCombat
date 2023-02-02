use ggez::{GameError, GameResult};
use inotify::{Inotify, WatchMask};

use crate::RESOURCE_PATH;

use super::Panel;

pub struct DebugGuiState {
    pub panel: Panel,
}

impl DebugGuiState {
    pub fn new() -> GameResult<Self> {
        GameResult::Ok(Self {
            panel: Panel::default(),
        })
    }
}
