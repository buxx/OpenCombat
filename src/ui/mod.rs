use ggez::graphics;

use crate::config::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH};
use crate::scene::item::SceneItem;
use crate::ui::scene_item_menu::SceneItemMenuItem;
use crate::WindowPoint;

pub mod scene_item_menu;

pub enum UiItem {
    SceneItemMenu,
}

pub struct UiSpriteInfo {
    pub relative_start_x: f32,
    pub relative_start_y: f32,
    pub relative_width: f32,
    pub relative_height: f32,
    pub width: f32,
    pub height: f32,
}

impl UiSpriteInfo {
    pub fn from_type(type_: UiItem) -> Self {
        match type_ {
            UiItem::SceneItemMenu => Self {
                relative_start_x: 0.0,
                relative_start_y: 0.0,
                relative_width: 71.0 / UI_SPRITE_SHEET_WIDTH,
                relative_height: 68.0 / UI_SPRITE_SHEET_HEIGHT,
                width: 71.0,
                height: 68.0,
            },
        }
    }

    pub fn as_draw_param(&self) -> graphics::DrawParam {
        graphics::DrawParam::new().src(graphics::Rect::new(
            self.relative_start_x,
            self.relative_start_y,
            self.relative_width,
            self.relative_height,
        ))
    }

    pub fn which_item_clicked(
        &self,
        _window_menu_point: WindowPoint,
        _window_click_point: WindowPoint,
        _scene_item: &SceneItem,
    ) -> Option<SceneItemMenuItem> {
        Some(SceneItemMenuItem::Move)
    }
}

#[derive(Debug)]
pub enum UserEvent {
    Click(WindowPoint),                      // Window coordinates
    RightClick(WindowPoint),                 // Window coordinates
    AreaSelection(WindowPoint, WindowPoint), // Window coordinates
}

pub enum SceneItemPrepareOrder {
    Move(usize), // scene_item usize
}
