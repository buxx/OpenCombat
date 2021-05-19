use ggez::graphics;

use crate::behavior::order::Order;
use crate::behavior::ItemBehavior;
use crate::config::{SCENE_ITEMS_SPRITE_SHEET_HEIGHT, SCENE_ITEMS_SPRITE_SHEET_WIDTH};
use crate::map::Map;
use crate::physics::GridPoint;
use crate::physics::{util, MetaEvent};
use crate::scene::SpriteType;
use crate::{Message, Offset, ScenePoint};

pub struct SceneItemSpriteInfo {
    pub relative_start_y: f32,
    pub relative_tile_width: f32,
    pub relative_tile_height: f32,
    pub tile_count: u16,
    pub tile_width: f32,
    pub tile_height: f32,
    pub _half_tile_width: f32,
    pub _half_tile_height: f32,
    pub tick_speed: f32,
}

impl SceneItemSpriteInfo {
    // TODO: ask on rust community if this is performant, or how to make it static
    pub fn from_type(type_: &SpriteType) -> Self {
        let (start_y, tile_width, tile_height, tile_count, tick_speed) = match type_ {
            SpriteType::WalkingSoldier => (12.0, 12.0, 12.0, 8, 0.5),
            SpriteType::CrawlingSoldier => (26.0, 26.0, 26.0, 8, 1.0),
            SpriteType::StandingSoldier => (0.0, 12.0, 12.0, 1, 0.0),
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
            tick_speed,
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
    pub grid_position: GridPoint,
    pub state: ItemState,
    pub meta_events: Vec<MetaEvent>,
    pub current_frame: f32,
    pub current_order: Option<Order>,
    pub next_order: Option<Order>,
    pub display_angle: f32,
}

impl SceneItem {
    pub fn new(type_: SceneItemType, position: ScenePoint, state: ItemState, map: &Map) -> Self {
        Self {
            type_,
            position: position.clone(),
            grid_position: util::grid_point_from_scene_point(&position.clone(), map),
            state,
            meta_events: vec![],
            current_frame: 0.0,
            current_order: None,
            next_order: None,
            display_angle: 0.0,
        }
    }

    pub fn sprite_info(&self) -> SceneItemSpriteInfo {
        SceneItemSpriteInfo::from_type(&self.sprite_type())
    }

    pub fn tick_sprite(&mut self) {
        let sprite_info = self.sprite_info();
        self.current_frame += sprite_info.tick_speed;
        // TODO: good way to have sprite info ? performant ?
        if self.current_frame as u16 >= sprite_info.tile_count {
            self.current_frame = 0.0;
        }
    }

    pub fn as_draw_param(&self, current_frame: f32) -> graphics::DrawParam {
        let sprite_info = self.sprite_info();
        graphics::DrawParam::new()
            .src(graphics::Rect::new(
                (current_frame as u16) as f32 * sprite_info.relative_tile_width,
                sprite_info.relative_start_y,
                sprite_info.relative_tile_width,
                sprite_info.relative_tile_height,
            ))
            .rotation(self.display_angle)
            .offset(Offset::new(0.5, 0.5))
    }

    pub fn sprite_type(&self) -> SpriteType {
        // Here some logical about state, nature (soldier, tank, ...) and current behavior to
        // determine sprite type
        match self.state.current_behavior {
            ItemBehavior::HideTo(_, _) => SpriteType::CrawlingSoldier,
            ItemBehavior::MoveTo(_, _) => SpriteType::WalkingSoldier,
            ItemBehavior::MoveFastTo(_, _) => SpriteType::WalkingSoldier,
            ItemBehavior::Standing => SpriteType::StandingSoldier,
        }
    }
}

pub enum SceneItemModifier {
    SwitchToNextOrder,
    ChangeDisplayAngle(f32),
    ChangeState(ItemState),
    ChangePosition(ScenePoint),
    ChangeGridPosition(GridPoint),
    ReachMoveGridPoint,
}

pub fn apply_scene_item_modifiers(
    scene_item: &mut SceneItem,
    modifiers: Vec<SceneItemModifier>,
) -> Vec<Message> {
    let mut messages: Vec<Message> = vec![];

    for modifier in modifiers.into_iter() {
        messages.extend(apply_scene_item_modifier(scene_item, modifier))
    }

    messages
}

pub fn apply_scene_item_modifier(
    scene_item: &mut SceneItem,
    modifier: SceneItemModifier,
) -> Vec<Message> {
    let mut messages: Vec<Message> = vec![];

    match modifier {
        SceneItemModifier::SwitchToNextOrder => {
            if let Some(next_order) = &scene_item.next_order {
                scene_item.current_order = Some(next_order.clone());
                scene_item.next_order = None;
            } else {
                scene_item.current_order = None;
                // TODO: Depending to context
                scene_item.state = ItemState::new(ItemBehavior::Standing);
            }
        }
        SceneItemModifier::ChangeDisplayAngle(new_angle) => {
            scene_item.display_angle = new_angle;
        }
        SceneItemModifier::ChangeState(new_state) => {
            scene_item.state = new_state;
        }
        SceneItemModifier::ChangePosition(new_point) => {
            scene_item.position = new_point;
        }
        SceneItemModifier::ChangeGridPosition(new_grid_point) => {
            scene_item.grid_position = new_grid_point;
        }
        SceneItemModifier::ReachMoveGridPoint => match &mut scene_item.state.current_behavior {
            ItemBehavior::Standing => {}
            ItemBehavior::HideTo(_, grid_path)
            | ItemBehavior::MoveTo(_, grid_path)
            | ItemBehavior::MoveFastTo(_, grid_path) => {
                grid_path.drain(0..1);
                // If target reached
                if grid_path.len() == 0 {
                    apply_scene_item_modifier(scene_item, SceneItemModifier::SwitchToNextOrder);
                }
            }
        },
    }

    messages
}
