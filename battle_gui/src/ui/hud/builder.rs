use battle_core::{state::battle::BattleState, types::WindowPoint};

use crate::{
    engine::state::GuiState,
    ui::{
        component::background::HorizontalBackground,
        hud::{
            BACKGROUND_CENTER_HEIGHT, BACKGROUND_CENTER_WIDTH, BACKGROUND_LEFT_WIDTH,
            BACKGROUND_REL_CENTER_HEIGHT, BACKGROUND_REL_CENTER_START_X,
            BACKGROUND_REL_CENTER_START_Y, BACKGROUND_REL_CENTER_WIDTH, BACKGROUND_RIGHT_HEIGHT,
            BACKGROUND_RIGHT_WIDTH,
        },
    },
};

use super::{
    background::Background, Hud, BACKGROUND_LEFT_HEIGHT, BACKGROUND_REL_LEFT_HEIGHT,
    BACKGROUND_REL_LEFT_START_X, BACKGROUND_REL_LEFT_START_Y, BACKGROUND_REL_LEFT_WIDTH,
    BACKGROUND_REL_RIGHT_HEIGHT, BACKGROUND_REL_RIGHT_START_X, BACKGROUND_REL_RIGHT_START_Y,
    BACKGROUND_REL_RIGHT_WIDTH,
};

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
        Hud::new(self.background())
    }

    fn background(&self) -> Background {
        let background = HorizontalBackground {
            rel_left_start_x: BACKGROUND_REL_LEFT_START_X,
            rel_left_start_y: BACKGROUND_REL_LEFT_START_Y,
            rel_left_width: BACKGROUND_REL_LEFT_WIDTH,
            rel_left_height: BACKGROUND_REL_LEFT_HEIGHT,
            left_width: BACKGROUND_LEFT_WIDTH,
            left_height: BACKGROUND_LEFT_HEIGHT,
            rel_center_start_x: BACKGROUND_REL_CENTER_START_X,
            rel_center_start_y: BACKGROUND_REL_CENTER_START_Y,
            rel_center_width: BACKGROUND_REL_CENTER_WIDTH,
            rel_center_height: BACKGROUND_REL_CENTER_HEIGHT,
            center_width: BACKGROUND_CENTER_WIDTH,
            center_height: BACKGROUND_CENTER_HEIGHT,
            rel_right_start_x: BACKGROUND_REL_RIGHT_START_X,
            rel_right_start_y: BACKGROUND_REL_RIGHT_START_Y,
            rel_right_width: BACKGROUND_REL_RIGHT_WIDTH,
            rel_right_height: BACKGROUND_REL_RIGHT_HEIGHT,
            right_width: BACKGROUND_RIGHT_WIDTH,
            right_height: BACKGROUND_RIGHT_HEIGHT,
        };
        let background_sprites = background.sprites(self.point, self.width, self.height);
        Background::new(
            background_sprites,
            self.point.clone(),
            self.width,
            self.height,
        )
    }
}
