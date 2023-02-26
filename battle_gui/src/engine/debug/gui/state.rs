use battle_core::game::explosive::ExplosiveType;
use ggez::GameResult;

use super::Panel;

pub struct DebugGuiState {
    pub panel: Panel,
    pub explosive: ExplosiveType,
}

impl DebugGuiState {
    pub fn new() -> GameResult<Self> {
        GameResult::Ok(Self {
            panel: Panel::default(),
            explosive: ExplosiveType::FA19241927,
        })
    }
}
