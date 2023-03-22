use battle_core::{
    audio::Sound,
    config::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH},
    types::WindowPoint,
};
use ggez::graphics::{Color, DrawParam, Text, TextFragment, TextLayout};

use crate::ui::component::{button::Button as UiButton, Component};

use super::event::HudEvent;

pub const BUTTON_START_X: f32 = 0.;
pub const BUTTON_START_Y: f32 = 450.;
pub const BUTTON_WIDTH: f32 = 71.;
pub const BUTTON_HEIGHT: f32 = 16.;

pub const BUTTON_REL_START_X: f32 = BUTTON_START_X / UI_SPRITE_SHEET_WIDTH;
pub const BUTTON_REL_START_Y: f32 = BUTTON_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const BUTTON_REL_WIDTH: f32 = BUTTON_WIDTH / UI_SPRITE_SHEET_WIDTH;
pub const BUTTON_REL_HEIGHT: f32 = BUTTON_HEIGHT / UI_SPRITE_SHEET_HEIGHT;

pub struct Button {
    text: String,
    action: Option<HudEvent>,
    enabled: bool,
    point: WindowPoint,
}

impl Button {
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

    pub fn center(&self) -> WindowPoint {
        WindowPoint::new(
            self.point.x + self.width() / 2.,
            self.point.y + self.height() / 2.,
        )
    }
}

impl Component<HudEvent> for Button {
    fn point(&self) -> WindowPoint {
        self.point
    }

    fn width(&self) -> f32 {
        BUTTON_WIDTH
    }

    fn height(&self) -> f32 {
        BUTTON_HEIGHT
    }

    fn contains(&self, points: &Vec<&WindowPoint>) -> bool {
        points.iter().all(|point| {
            point.x >= self.point.x
                && point.x <= self.point.x + self.width()
                && point.y >= self.point.y
                && point.y <= self.point.y + self.height()
        })
    }

    fn sprites(&self, hovered: &WindowPoint) -> Vec<DrawParam> {
        UiButton {
            rel_start_x: BUTTON_REL_START_X,
            rel_start_y: BUTTON_REL_START_Y,
            rel_width: BUTTON_REL_WIDTH,
            rel_height: BUTTON_REL_HEIGHT,
        }
        .sprites(self.point, self.enabled, self.contains(&vec![hovered]))
    }

    fn event(&self) -> Option<HudEvent> {
        self.action.clone()
    }

    fn sound(&self) -> Option<Sound> {
        if self.enabled {
            return Some(Sound::Clic1);
        }

        None
    }

    fn draw(&self, _hovered: &WindowPoint, canvas: &mut ggez::graphics::Canvas) {
        canvas.draw(
            Text::new(TextFragment::new(&self.text).color(Color::WHITE))
                .set_layout(TextLayout::center())
                .set_bounds(self.bounds()),
            DrawParam::default().dest(self.center().to_vec2()),
        )
    }
}
