use ggez::graphics;

use crate::config::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH};
use crate::scene::item::SceneItem;
use crate::{Offset, ScenePoint, WindowPoint};

pub mod vertical_menu;

const SCENE_ITEM_MENU_WIDTH: f32 = 71.0;
const SCENE_ITEM_MENU_HEIGHT: f32 = 68.0;
const SCENE_ITEM_MENU_ITEM_HEIGHT: f32 = 15.0;

pub enum UiComponent {
    SceneItemMenu,
}

#[derive(Debug)]
pub enum UserEvent {
    Click(WindowPoint),
    RightClick(WindowPoint),
    AreaSelection(WindowPoint, WindowPoint),
}

pub enum SceneItemPrepareOrder {
    Move(usize), // scene_item usize
}

#[derive(Clone)]
pub enum MenuItem {
    Move,
    MoveFast,
    Hide,
}
