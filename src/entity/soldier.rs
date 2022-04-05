use crate::{
    behavior::Behavior,
    config::{SOLDIER_SELECTABLE_SQUARE_SIDE, SOLDIER_SELECTABLE_SQUARE_SIDE_HALF},
    game::Side,
    graphics::{animation::Sprite, soldier::SoldierAnimationType},
    types::*,
};
use ggez::graphics::Rect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Soldier {
    side: Side,
    world_point: WorldPoint,
    squad_uuid: SquadUuid,
    behavior: Behavior,
    looking_direction: Angle,
}

impl Soldier {
    pub fn new(world_point: WorldPoint, squad_uuid: SquadUuid, side: Side) -> Self {
        Self {
            side,
            world_point,
            squad_uuid,
            behavior: Behavior::Idle,
            looking_direction: Angle(0.0),
        }
    }

    pub fn from_soldier(soldier: &Soldier) -> Self {
        Self::new(
            soldier.get_world_point(),
            soldier.squad_uuid(),
            *soldier.get_side(),
        )
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

    pub fn set_behavior(&mut self, behavior: Behavior) {
        self.behavior = behavior
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
        };
        Box::new(animation_type)
    }
}
