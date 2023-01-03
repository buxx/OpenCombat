use std::cmp::min;

use crate::{
    behavior::{feeling::Feeling, Behavior},
    config::{SOLDIER_SELECTABLE_SQUARE_SIDE, SOLDIER_SELECTABLE_SQUARE_SIDE_HALF},
    game::Side,
    graphics::{animation::Sprite, soldier::SoldierAnimationType},
    order::Order,
    types::*,
};
use ggez::graphics::Rect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Soldier {
    uuid: SoldierIndex,
    side: Side,
    world_point: WorldPoint,
    squad_uuid: SquadUuid,
    order: Option<Order>,
    behavior: Behavior,
    looking_direction: Angle,
    alive: bool,
    unconscious: bool,
    under_fire: Feeling,
}

impl Soldier {
    pub fn new(
        uuid: SoldierIndex,
        world_point: WorldPoint,
        squad_uuid: SquadUuid,
        side: Side,
    ) -> Self {
        Self {
            uuid,
            side,
            world_point,
            squad_uuid,
            order: None,
            behavior: Behavior::Idle,
            looking_direction: Angle(0.0),
            alive: true,
            unconscious: false,
            under_fire: Feeling::UnderFire(0),
        }
    }

    pub fn from_soldier(soldier: &Soldier) -> Self {
        Self::new(
            soldier.uuid(),
            soldier.get_world_point(),
            soldier.squad_uuid(),
            *soldier.get_side(),
        )
    }

    pub fn uuid(&self) -> SoldierIndex {
        self.uuid
    }

    pub fn get_side(&self) -> &Side {
        &self.side
    }

    pub fn get_world_point(&self) -> WorldPoint {
        self.world_point
    }

    pub fn set_world_point(&mut self, point: WorldPoint) {
        self.world_point = point
    }

    pub fn squad_uuid(&self) -> SquadUuid {
        self.squad_uuid
    }

    pub fn get_behavior(&self) -> &Behavior {
        &self.behavior
    }

    pub fn get_behavior_mut(&mut self) -> &mut Behavior {
        &mut self.behavior
    }

    pub fn order(&self) -> Option<&Order> {
        match &self.order {
            Some(order) => Some(order),
            None => None,
        }
    }

    pub fn order_mut(&mut self) -> &mut Option<Order> {
        &mut self.order
    }

    pub fn set_behavior(&mut self, behavior: Behavior) {
        self.behavior = behavior
    }

    pub fn set_order(&mut self, order: Option<Order>) {
        self.order = order
    }

    pub fn get_looking_direction(&self) -> Angle {
        self.looking_direction
    }

    pub fn set_looking_direction(&mut self, angle: Angle) {
        self.looking_direction = angle
    }

    pub fn get_selection_rect(&self) -> Rect {
        Rect::new(
            self.world_point.x - SOLDIER_SELECTABLE_SQUARE_SIDE_HALF,
            self.world_point.y - SOLDIER_SELECTABLE_SQUARE_SIDE_HALF,
            SOLDIER_SELECTABLE_SQUARE_SIDE,
            SOLDIER_SELECTABLE_SQUARE_SIDE,
        )
    }

    pub fn get_animation_type(&self) -> Box<dyn Sprite> {
        let animation_type = match self.get_behavior() {
            Behavior::Idle => SoldierAnimationType::Idle,
            Behavior::MoveTo(_) => SoldierAnimationType::Walking,
            Behavior::MoveFastTo(_) => SoldierAnimationType::Walking,
            Behavior::SneakTo(_) => SoldierAnimationType::Crawling,
            Behavior::Defend(_) => SoldierAnimationType::LyingDown,
            Behavior::Hide(_) => SoldierAnimationType::LyingDown,
            Behavior::CommandDriveTo(_) => SoldierAnimationType::Idle,
            Behavior::CommandRotateTo(_) => SoldierAnimationType::Idle,
            Behavior::DriveTo(_) => SoldierAnimationType::Idle,
            Behavior::RotateTo(_) => SoldierAnimationType::Idle,
            // TODO : Different animation according to death type
            Behavior::Dead => SoldierAnimationType::DeadWithSideBlood,
            Behavior::Unconscious => SoldierAnimationType::LyingDown,
        };
        Box::new(animation_type)
    }

    pub fn set_alive(&mut self, value: bool) {
        self.alive = value
    }

    pub fn set_unconscious(&mut self, value: bool) {
        self.unconscious = value
    }

    pub fn can_be_animated(&self) -> bool {
        self.alive && !self.unconscious
    }

    pub fn can_produce_sound(&self) -> bool {
        self.alive && !self.unconscious
    }

    pub fn can_feel_explosion(&self) -> bool {
        self.alive
    }

    pub fn can_see_interior(&self) -> bool {
        self.alive && !self.unconscious
    }

    pub fn can_seek(&self) -> bool {
        self.alive && !self.unconscious
    }

    pub fn under_fire(&self) -> &Feeling {
        &self.under_fire
    }

    pub fn increase_under_fire(&mut self, value: u32) {
        self.under_fire.increase(value)
    }

    pub fn decrease_under_fire(&mut self) {
        self.under_fire.decrease()
    }
}
