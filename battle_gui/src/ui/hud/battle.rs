use battle_core::{audio::Sound, types::WindowPoint};
use ggez::{
    graphics::{Color, DrawParam, Text, TextFragment, TextLayout},
    Context, GameResult,
};
use oc_core::graphics::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH};

use crate::ui::component::{button::Button as UiButton, Component};

use super::event::HudEvent;

pub const BATTLE_BUTTON_START_X: f32 = 0.;
pub const BATTLE_BUTTON_START_Y: f32 = 450.;
pub const BATTLE_BUTTON_WIDTH: f32 = 71.;
pub const BATTLE_BUTTON_HEIGHT: f32 = 16.;

pub const BATTLE_BUTTON_REL_START_X: f32 = BATTLE_BUTTON_START_X / UI_SPRITE_SHEET_WIDTH;
pub const BATTLE_BUTTON_REL_START_Y: f32 = BATTLE_BUTTON_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const BATTLE_BUTTON_REL_WIDTH: f32 = BATTLE_BUTTON_WIDTH / UI_SPRITE_SHEET_WIDTH;
pub const BATTLE_BUTTON_REL_HEIGHT: f32 = BATTLE_BUTTON_HEIGHT / UI_SPRITE_SHEET_HEIGHT;

pub struct BattleButton {
    text: String,
    action: Option<HudEvent>,
    enabled: bool,
    point: WindowPoint,
}

impl BattleButton {
    pub fn begin(point: WindowPoint, enabled: bool) -> Self {
        if enabled {
            Self {
                text: "Begin".to_string(),
                action: Some(HudEvent::RequestBeginBattle),
                enabled,
                point,
            }
        } else {
            Self {
                text: "Begin".to_string(),
                action: None,
                enabled,
                point,
            }
        }
    }

    pub fn end(point: WindowPoint, enabled: bool) -> Self {
        if enabled {
            Self {
                text: "End".to_string(),
                action: Some(HudEvent::RequestEndBattle),
                enabled,
                point,
            }
        } else {
            Self {
                text: "End".to_string(),
                action: None,
                enabled,
                point,
            }
        }
    }
}

impl Component<HudEvent> for BattleButton {
    fn point(&self, _ctx: &Context) -> WindowPoint {
        self.point
    }

    fn width(&self, _ctx: &Context) -> f32 {
        BATTLE_BUTTON_WIDTH
    }

    fn height(&self, _ctx: &Context) -> f32 {
        BATTLE_BUTTON_HEIGHT
    }

    fn sprites(&self, ctx: &Context, hovered: &WindowPoint) -> Vec<DrawParam> {
        UiButton {
            rel_start_x: BATTLE_BUTTON_REL_START_X,
            rel_start_y: BATTLE_BUTTON_REL_START_Y,
            rel_width: BATTLE_BUTTON_REL_WIDTH,
            rel_height: BATTLE_BUTTON_REL_HEIGHT,
        }
        .sprites(self.point, self.enabled, self.contains(ctx, &vec![hovered]))
    }

    fn event(&self, _ctx: &Context) -> Option<HudEvent> {
        self.action.clone()
    }

    fn sound(&self, _ctx: &Context) -> Option<Sound> {
        if self.enabled {
            return Some(Sound::Clic1);
        }

        None
    }

    fn draw(
        &self,
        ctx: &mut Context,
        _hovered: &WindowPoint,
        canvas: &mut ggez::graphics::Canvas,
    ) -> GameResult {
        canvas.draw(
            Text::new(TextFragment::new(&self.text).color(Color::WHITE))
                .set_layout(TextLayout::center())
                .set_bounds(self.bounds(ctx)),
            DrawParam::default().dest(self.center(ctx).to_vec2()),
        );

        Ok(())
    }
}
