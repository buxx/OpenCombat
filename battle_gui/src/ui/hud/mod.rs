use self::{
    background::Background, battle::BattleButton, event::HudEvent, morale::MoraleIndicator,
};
use battle_core::types::WindowPoint;

use super::component::Component;

pub mod background;
pub mod battle;
pub mod builder;
pub mod event;
pub mod morale;
pub mod painter;

pub const HUD_HEIGHT: f32 = 200.0;

pub struct Hud {
    background: Background,
    battle_button: BattleButton,
    morale_indicator: MoraleIndicator,
}

impl Hud {
    pub fn new(
        background: Background,
        battle: BattleButton,
        morale_indicator: MoraleIndicator,
    ) -> Self {
        Self {
            background,
            battle_button: battle,
            morale_indicator,
        }
    }

    pub fn background(&self) -> &Background {
        &self.background
    }

    pub fn battle_button(&self) -> &BattleButton {
        &self.battle_button
    }

    pub fn morale_indicator(&self) -> &MoraleIndicator {
        &self.morale_indicator
    }

    pub fn contains(&self, points: &Vec<&WindowPoint>) -> bool {
        self.background.contains(points)
    }

    pub fn hovered_by(&self, points: &Vec<&WindowPoint>) -> Option<Box<&dyn Component<HudEvent>>> {
        if !self.contains(points) {
            return None;
        }

        if self.battle_button.contains(points) {
            return Some(Box::new(&self.battle_button));
        }

        // TODO : parse different components like squad icon, etc
        if self.background.contains(points) {
            return Some(Box::new(&self.background));
        }

        None
    }
}
