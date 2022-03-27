use ggez::graphics::Rect;

use crate::{behavior::Behavior, game::Side, graphics::animation::Sprite, types::*};
pub mod soldier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Soldier,
}

pub trait Entity {
    fn get_side(&self) -> &Side;
    fn get_world_point(&self) -> WorldPoint;
    fn set_world_point(&mut self, point: WorldPoint);
    fn get_behavior(&self) -> &Behavior;
    fn get_behavior_mut(&mut self) -> &mut Behavior;
    fn set_behavior(&mut self, behavior: Behavior);
    fn get_looking_direction(&self) -> Angle;
    fn set_looking_direction(&mut self, angle: Angle);
    fn squad_uuid(&self) -> SquadUuid;
    fn get_type(&self) -> EntityType;
    fn get_selection_rect(&self) -> Rect;
    fn get_animation_type(&self) -> Box<dyn Sprite>;
}
