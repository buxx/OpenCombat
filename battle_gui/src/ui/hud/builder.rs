use battle_core::{
    state::battle::{phase::Phase, BattleState},
    types::WindowPoint,
};
use glam::Vec2;

use crate::engine::state::GuiState;

use super::{background::Background, button::Button, Hud};

const MARGIN: f32 = 5.;
const RIGHT_BOX_WIDTH: f32 = 200.;

pub struct HudBuilder<'a> {
    gui_state: &'a GuiState,
    battle_state: &'a BattleState,
    point: WindowPoint,
    width: f32,
    height: f32,
}

impl<'a> HudBuilder<'a> {
    pub fn new(gui_state: &'a GuiState, battle_state: &'a BattleState) -> Self {
        Self {
            gui_state,
            battle_state,
            point: WindowPoint::new(0., 0.),
            width: 0.,
            height: 0.,
        }
    }

    pub fn point(mut self, point: WindowPoint) -> Self {
        self.point = point;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn build(&self) -> Hud {
        Hud::new(
            Background::new(self.point.clone(), self.width, self.height),
            self.battle_button(),
        )
    }

    fn battle_button(&self) -> Button {
        let point = self
            .point
            .apply(Vec2::new(self.width - RIGHT_BOX_WIDTH, MARGIN));
        match self.battle_state.phase() {
            Phase::Placement => {
                let enabled = !self.battle_state.ready(self.gui_state.side());
                Button::begin(point, enabled)
            }
            // FIXME BS NOW : enabled computing
            Phase::Battle => Button::end(point, true),
            Phase::End => Button::end(point, false),
        }
    }
}
