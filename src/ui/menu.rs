use ggez::graphics;

use crate::{
    config::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH},
    types::*,
};

const SCENE_ITEM_MENU_WIDTH: f32 = 71.0;
const SCENE_ITEM_MENU_HEIGHT: f32 = 90.0;
const SCENE_ITEM_MENU_ITEM_HEIGHT: f32 = 15.0;

#[derive(Clone, Debug)]
pub enum MenuItem {
    Move,
    MoveFast,
    Sneak,
    Defend,
    Hide,
}

pub fn squad_menu_sprite_info() -> VerticalMenuSpriteInfo {
    VerticalMenuSpriteInfo {
        relative_start_x: 0.0,
        relative_start_y: 0.0,
        relative_width: SCENE_ITEM_MENU_WIDTH / UI_SPRITE_SHEET_WIDTH,
        relative_height: SCENE_ITEM_MENU_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
        width: SCENE_ITEM_MENU_WIDTH,
        height: SCENE_ITEM_MENU_HEIGHT,
        item_height: SCENE_ITEM_MENU_ITEM_HEIGHT,
        relative_item_height: SCENE_ITEM_MENU_ITEM_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
        item_matches: vec![
            MenuItem::Move,
            MenuItem::MoveFast,
            MenuItem::Sneak,
            MenuItem::Defend,
            MenuItem::Defend,
            MenuItem::Hide,
        ],
    }
}

pub struct VerticalMenuSpriteInfo {
    pub relative_start_x: f32,
    pub relative_start_y: f32,
    pub relative_width: f32,
    pub relative_height: f32,
    pub width: f32,
    pub height: f32,
    pub item_height: f32,
    pub relative_item_height: f32,
    pub item_matches: Vec<MenuItem>,
}

impl VerticalMenuSpriteInfo {
    fn item_position(
        &self,
        menu_scene_point: &WindowPoint,
        current_cursor_scene_point: &WindowPoint,
    ) -> Option<usize> {
        let relative_cursor_position: Offset = Offset::new(
            current_cursor_scene_point.x - menu_scene_point.x,
            current_cursor_scene_point.y - menu_scene_point.y,
        );

        // Cursor inside menu
        if relative_cursor_position.x >= 0.0
            && relative_cursor_position.x <= SCENE_ITEM_MENU_WIDTH
            && relative_cursor_position.y >= 0.0
            && relative_cursor_position.y <= SCENE_ITEM_MENU_HEIGHT
        {
            return Some((relative_cursor_position.y / SCENE_ITEM_MENU_ITEM_HEIGHT) as usize);
        }

        None
    }

    pub fn as_draw_params(
        &self,
        menu_scene_point: &WindowPoint,
        scene_current_cursor_point: &WindowPoint,
    ) -> Vec<graphics::DrawParam> {
        let mut draw_params = vec![graphics::DrawParam::new()
            .src(graphics::Rect::new(
                self.relative_start_x,
                self.relative_start_y,
                self.relative_width,
                self.relative_height,
            ))
            .dest(menu_scene_point.to_vec2())];

        if let Some(item_position) =
            self.item_position(menu_scene_point, scene_current_cursor_point)
        {
            let source = graphics::Rect::new(
                self.relative_width,
                self.relative_item_height * item_position as f32,
                self.relative_width,
                self.relative_item_height,
            );
            let destination = WindowPoint::new(
                menu_scene_point.x,
                menu_scene_point.y + (self.item_height * item_position as f32),
            );
            draw_params.push(
                graphics::DrawParam::new()
                    .src(source)
                    .dest(destination.to_vec2()),
            );
        }

        draw_params
    }

    pub fn item_clicked(
        &self,
        menu_scene_point: &WindowPoint,
        scene_current_cursor_point: &WindowPoint,
    ) -> Option<MenuItem> {
        if let Some(item_position) =
            self.item_position(menu_scene_point, scene_current_cursor_point)
        {
            if let Some(menu_item) = self.item_matches.get(item_position) {
                return Some(menu_item.clone());
            }
        };

        None
    }
}
