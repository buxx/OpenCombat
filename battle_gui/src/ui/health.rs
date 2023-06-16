use battle_core::game::health::Health;

use crate::utils::{GREEN, RED, YELLOW};

use super::color::Colorized;

impl Colorized for Health {
    fn color(&self) -> ggez::graphics::Color {
        match self {
            Health::Good => GREEN,
            Health::Unconscious => YELLOW,
            Health::Dead => RED,
        }
    }
}
