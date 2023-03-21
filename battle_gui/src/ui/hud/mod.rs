use self::{background::Background, button::Button, event::HudEvent};
use battle_core::types::WindowPoint;

use super::component::Component;

pub mod background;
pub mod builder;
pub mod button;
pub mod event;
pub mod painter;

pub const HUD_HEIGHT: f32 = 200.0;

pub struct Hud {
    background: Background,
    battle: Button,
}

impl Hud {
    pub fn new(background: Background, battle: Button) -> Self {
        Self { background, battle }
    }

    pub fn background(&self) -> &Background {
        &self.background
    }

    pub fn battle(&self) -> &Button {
        &self.battle
    }

    pub fn contains(&self, points: &Vec<&WindowPoint>) -> bool {
        self.background.contains(points)
    }

    pub fn hovered_by(&self, points: &Vec<&WindowPoint>) -> Option<Box<&dyn Component<HudEvent>>> {
        if !self.contains(points) {
            return None;
        }

        if self.battle.contains(points) {
            return Some(Box::new(&self.battle));
        }

        // TODO : parse different components like squad icon, etc
        if self.background.contains(points) {
            return Some(Box::new(&self.background));
        }

        None
    }
}
