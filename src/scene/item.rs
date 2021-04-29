use ggez::graphics;

use crate::behavior::ItemBehavior;
use crate::config::{SCENE_ITEMS_SPRITE_SHEET_HEIGHT, SCENE_ITEMS_SPRITE_SHEET_WIDTH};
use crate::physics::GridPosition;
use crate::physics::{util, MetaEvent};
use crate::scene::SpriteType;
use crate::{Offset, ScenePoint};

pub struct SceneItemSpriteInfo {
    pub relative_start_y: f32,
    pub relative_tile_width: f32,
    pub relative_tile_height: f32,
    pub tile_count: u16,
    pub tile_width: f32,
    pub tile_height: f32,
    pub _half_tile_width: f32,
    pub _half_tile_height: f32,
}

impl SceneItemSpriteInfo {
    // TODO: ask on rust community if this is performant, or how to make it static
    pub fn from_type(type_: &SpriteType) -> Self {
        let (start_y, tile_width, tile_height, tile_count) = match type_ {
            SpriteType::WalkingSoldier => (12.0, 12.0, 12.0, 8),
            SpriteType::CrawlingSoldier => (26.0, 26.0, 26.0, 8),
            SpriteType::StandingSoldier => (0.0, 12.0, 12.0, 1),
        };

        Self {
            relative_start_y: start_y / SCENE_ITEMS_SPRITE_SHEET_HEIGHT,
            relative_tile_width: tile_width / SCENE_ITEMS_SPRITE_SHEET_WIDTH,
            relative_tile_height: tile_height / SCENE_ITEMS_SPRITE_SHEET_HEIGHT,
            tile_count,
            tile_width,
            tile_height,
            _half_tile_width: tile_width / 2.0,
            _half_tile_height: tile_height / 2.0,
        }
    }
}

pub struct ItemState {
    pub current_behavior: ItemBehavior,
}

impl ItemState {
    pub fn new(current_behavior: ItemBehavior) -> Self {
        Self { current_behavior }
    }
}

pub enum SceneItemType {
    Soldier,
}

pub struct SceneItem {
    pub type_: SceneItemType,
    pub position: ScenePoint,
    pub grid_position: GridPosition,
    pub state: ItemState,
    pub meta_events: Vec<MetaEvent>,
    pub current_frame: u16,
}

impl SceneItem {
    pub fn new(type_: SceneItemType, position: ScenePoint, state: ItemState) -> Self {
        Self {
            type_,
            position: position.clone(),
            grid_position: util::grid_position_from_scene_point(&position.clone()),
            state,
            meta_events: vec![],
            current_frame: 0,
        }
    }

    pub fn sprite_info(&self) -> SceneItemSpriteInfo {
        SceneItemSpriteInfo::from_type(&self.sprite_type())
    }

    pub fn tick_sprite(&mut self) {
        self.current_frame += 1;
        // TODO: good way to have sprite info ? performant ?
        if self.current_frame >= self.sprite_info().tile_count {
            self.current_frame = 0;
        }
    }

    pub fn as_draw_param(&self, current_frame: f32) -> graphics::DrawParam {
        let sprite_info = self.sprite_info();
        graphics::DrawParam::new()
            .src(graphics::Rect::new(
                current_frame as f32 * sprite_info.relative_tile_width,
                sprite_info.relative_start_y,
                sprite_info.relative_tile_width,
                sprite_info.relative_tile_height,
            ))
            .rotation(90.0f32.to_radians())
            .offset(Offset::new(0.5, 0.5))
    }

    pub fn sprite_type(&self) -> SpriteType {
        // Here some logical about state, nature (soldier, tank, ...) and current behavior to
        // determine sprite type
        match self.state.current_behavior {
            ItemBehavior::Crawling => SpriteType::CrawlingSoldier,
            ItemBehavior::Walking(_) => SpriteType::WalkingSoldier,
            ItemBehavior::Standing(_) => SpriteType::StandingSoldier,
        }
    }
}
