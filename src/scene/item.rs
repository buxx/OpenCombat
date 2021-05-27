use ggez::graphics;

use crate::audio::Sound;
use crate::behavior::order::Order;
use crate::behavior::ItemBehavior;
use crate::config::{SCENE_ITEMS_SPRITE_SHEET_HEIGHT, SCENE_ITEMS_SPRITE_SHEET_WIDTH};
use crate::gameplay::weapon::Weapon;
use crate::map::Map;
use crate::physics::visibility::Visibility;
use crate::physics::{util, MetaEvent};
use crate::physics::{GridPoint, PhysicEvent};
use crate::scene::main::MainStateModifier;
use crate::scene::SpriteType;
use crate::{FrameI, Message, Offset, SceneItemId, ScenePoint};

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
            SpriteType::EngagingSoldier => (52.0, 26.0, 26.0, 1, 0.0),
            SpriteType::DeadSoldier => (78.0, 26.0, 26.0, 1, 0.0),
            SpriteType::UnconsciousSoldier => (78.0, 26.0, 26.0, 1, 0.0),
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

pub enum SceneItemType {
    Soldier,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Side {
    A,
    B,
}

pub struct SceneItem {
    pub id: usize,
    pub type_: SceneItemType,
    pub position: ScenePoint,
    pub grid_position: GridPoint,
    pub behavior: ItemBehavior,
    pub meta_events: Vec<MetaEvent>,
    pub current_frame: f32,
    pub current_order: Option<Order>,
    pub next_order: Option<Order>,
    pub display_angle: f32,
    pub visibilities: Vec<Visibility>,
    pub side: Side,
    pub weapon: Weapon,
    pub reloading_since: Option<FrameI>,
    pub acquiring_until: Option<FrameI>,
    pub alive: bool,
    pub incapacity: bool,
    pub last_bullet_fire: Option<FrameI>,
}

impl SceneItem {
    pub fn new(
        id: usize,
        type_: SceneItemType,
        position: ScenePoint,
        behavior: ItemBehavior,
        map: &Map,
        side: Side,
        weapon: Weapon,
    ) -> Self {
        Self {
            id,
            type_,
            position: position.clone(),
            grid_position: util::grid_point_from_scene_point(&position.clone(), map),
            behavior,
            meta_events: vec![],
            current_frame: 0.0,
            current_order: None,
            next_order: None,
            display_angle: 0.0,
            visibilities: vec![],
            side,
            weapon,
            reloading_since: None,
            acquiring_until: None,
            alive: true,
            incapacity: false,
            last_bullet_fire: None,
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
        match self.behavior {
            ItemBehavior::HideTo(_, _) => SpriteType::CrawlingSoldier,
            ItemBehavior::MoveTo(_, _) => SpriteType::WalkingSoldier,
            ItemBehavior::MoveFastTo(_, _) => SpriteType::WalkingSoldier,
            ItemBehavior::Standing => SpriteType::StandingSoldier,
            ItemBehavior::EngageSceneItem(_) => SpriteType::EngagingSoldier,
            ItemBehavior::EngageGridPoint(_) => SpriteType::EngagingSoldier,
            ItemBehavior::Dead => SpriteType::UnconsciousSoldier,
            ItemBehavior::Unconscious => SpriteType::DeadSoldier,
            ItemBehavior::Hide => SpriteType::EngagingSoldier,
        }
    }

    pub fn visible_scene_items_visibilities(&self) -> Vec<&Visibility> {
        self.visibilities
            .iter()
            .filter(|v| v.visible && v.to_scene_item_id.is_some())
            .collect::<Vec<&Visibility>>()
    }

    pub fn visible_scene_items_visibilities_for(
        &self,
        scene_item_id: SceneItemId,
    ) -> Option<&Visibility> {
        for visibility in self.visible_scene_items_visibilities().iter() {
            if visibility.to_scene_item_id.expect("visible_scene_items_visibilities must return only visibilities with to_scene_item_id") == scene_item_id {
                return Some(visibility);
            }
        }

        None
    }
}

pub enum SceneItemModifier {
    SwitchToNextOrder,
    ChangeDisplayAngle(f32),
    ChangeBehavior(ItemBehavior),
    ChangePosition(ScenePoint),
    ChangeGridPosition(GridPoint),
    ReachMoveGridPoint,
    ChangeVisibilities(Vec<Visibility>),
    SetNextOrder(Order),
    AcquireUntil(FrameI), // until frame_i
    FireOnSceneItem(Visibility),
    Disengage,
    Death,
    Incapacity,
    SetLastBulletFire(FrameI),
}

pub fn apply_scene_item_modifiers(
    frame_i: FrameI,
    scene_item: &mut SceneItem,
    modifiers: Vec<SceneItemModifier>,
) -> Vec<Message> {
    let mut messages: Vec<Message> = vec![];

    for modifier in modifiers.into_iter() {
        messages.extend(apply_scene_item_modifier(frame_i, scene_item, modifier))
    }

    messages
}

pub fn apply_scene_item_modifier(
    frame_i: FrameI,
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
                let new_behavior = if let Some(order) = &scene_item.current_order {
                    match order {
                        Order::HideTo(_) => ItemBehavior::Hide,
                        _ => ItemBehavior::Standing,
                    }
                } else {
                    ItemBehavior::Standing
                };

                scene_item.current_order = None;
                scene_item.behavior = new_behavior;
            }
        }
        SceneItemModifier::ChangeDisplayAngle(new_angle) => {
            scene_item.display_angle = new_angle;
        }
        SceneItemModifier::ChangeBehavior(new_behavior) => {
            scene_item.behavior = new_behavior;
        }
        SceneItemModifier::ChangePosition(new_point) => {
            scene_item.position = new_point;
        }
        SceneItemModifier::ChangeGridPosition(new_grid_point) => {
            scene_item.grid_position = new_grid_point;
        }
        SceneItemModifier::ReachMoveGridPoint => match &mut scene_item.behavior {
            ItemBehavior::Standing
            | ItemBehavior::Hide
            | ItemBehavior::Unconscious
            | ItemBehavior::Dead => {}
            ItemBehavior::EngageSceneItem(_) => {}
            ItemBehavior::EngageGridPoint(_) => {}
            ItemBehavior::HideTo(_, grid_path)
            | ItemBehavior::MoveTo(_, grid_path)
            | ItemBehavior::MoveFastTo(_, grid_path) => {
                grid_path.drain(0..1);
                // If target reached
                if grid_path.len() == 0 {
                    messages.extend(apply_scene_item_modifier(
                        frame_i,
                        scene_item,
                        SceneItemModifier::SwitchToNextOrder,
                    ));
                }
            }
        },
        SceneItemModifier::ChangeVisibilities(visibilities) => {
            scene_item.visibilities = visibilities
        }
        SceneItemModifier::SetNextOrder(order) => {
            scene_item.next_order = Some(order);
        }
        SceneItemModifier::AcquireUntil(until_frame_i) => {
            scene_item.acquiring_until = Some(until_frame_i)
        }
        SceneItemModifier::FireOnSceneItem(visibility) => {
            scene_item.acquiring_until = None;
            scene_item.weapon.need_reload = true;
            scene_item.reloading_since = Some(frame_i);
            messages.push(Message::MainStateMessage(
                MainStateModifier::PushPhysicEvent(PhysicEvent::BulletFire(visibility)),
            ))
        }
        SceneItemModifier::Disengage => {
            scene_item.behavior = ItemBehavior::Standing;
        }
        SceneItemModifier::Death => {
            scene_item.alive = false;
            scene_item.incapacity = true;
            scene_item.next_order = None;
            scene_item.current_order = None;
            scene_item.behavior = ItemBehavior::Unconscious;
        }
        SceneItemModifier::Incapacity => {
            scene_item.incapacity = true;
            scene_item.next_order = None;
            scene_item.current_order = None;
            scene_item.behavior = ItemBehavior::Unconscious;
        }
        SceneItemModifier::SetLastBulletFire(frame_i) => {
            scene_item.last_bullet_fire = Some(frame_i)
        }
    }

    messages
}
