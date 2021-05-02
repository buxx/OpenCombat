use ggez::graphics;

use crate::config::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH};
use crate::scene::item::SceneItem;
use crate::ui::scene_item_menu::SceneItemMenuItem;
use crate::{Offset, ScenePoint, WindowPoint};

pub mod scene_item_menu;

const SCENE_ITEM_MENU_WIDTH: f32 = 71.0;
const SCENE_ITEM_MENU_HEIGHT: f32 = 68.0;
const SCENE_ITEM_MENU_ITEM_HEIGHT: f32 = 15.0;

pub enum UiItem {
    SceneItemMenu,
}
// FIXME BS NOW: Transformer ça en VerticalMenu, où l'on donne les carac des item aussi
// --> ce sera capable de manière generique d'eclairer le bon item et dire quel item affiché
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
                relative_width: SCENE_ITEM_MENU_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: SCENE_ITEM_MENU_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: SCENE_ITEM_MENU_WIDTH,
                height: SCENE_ITEM_MENU_HEIGHT,
            },
        }
    }

    pub fn as_draw_params(
        &self,
        scene_point: &ScenePoint,
        scene_current_cursor_point: &ScenePoint,
    ) -> Vec<graphics::DrawParam> {
        // FIXME BS NOW: this is a generic struct ! not SceneItemMenu struct
        let mut draw_params = vec![graphics::DrawParam::new()
            .src(graphics::Rect::new(
                self.relative_start_x,
                self.relative_start_y,
                self.relative_width,
                self.relative_height,
            ))
            .dest(*scene_point)];

        let relative_cursor_position: Offset = Offset::new(
            scene_current_cursor_point.x - scene_point.x,
            scene_current_cursor_point.y - scene_point.y,
        );

        // Cursor inside menu
        if relative_cursor_position.x >= 0.0
            && relative_cursor_position.x <= SCENE_ITEM_MENU_WIDTH
            && relative_cursor_position.y >= 0.0
            && relative_cursor_position.y <= SCENE_ITEM_MENU_HEIGHT
        {
            let hover_item_i = (relative_cursor_position.y / SCENE_ITEM_MENU_ITEM_HEIGHT) as i32;
            let source = graphics::Rect::new(
                self.relative_width,
                (SCENE_ITEM_MENU_ITEM_HEIGHT / UI_SPRITE_SHEET_HEIGHT) * hover_item_i as f32,
                self.relative_width,
                SCENE_ITEM_MENU_ITEM_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
            );
            let destination = ScenePoint::new(
                scene_point.x,
                scene_point.y + (SCENE_ITEM_MENU_ITEM_HEIGHT * hover_item_i as f32),
            );
            draw_params.push(graphics::DrawParam::new().src(source).dest(destination));
        }

        draw_params
    }

    // FIXME BS NOW: this is a generic struct ! not SceneItemMenu struct
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
