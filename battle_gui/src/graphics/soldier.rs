use battle_core::{
    behavior::Behavior,
    config::{SOLDIER_SELECTABLE_SQUARE_SIDE, SOLDIER_SELECTABLE_SQUARE_SIDE_HALF},
    entity::soldier::Soldier,
    graphics::{soldier::SoldierAnimationType, Sprite},
};
use ggez::graphics::Rect;

use super::Graphics;

impl Graphics {
    pub fn soldier_selection_rect(&self, soldier: &Soldier) -> Rect {
        Rect::new(
            soldier.get_world_point().x - SOLDIER_SELECTABLE_SQUARE_SIDE_HALF,
            soldier.get_world_point().y - SOLDIER_SELECTABLE_SQUARE_SIDE_HALF,
            SOLDIER_SELECTABLE_SQUARE_SIDE,
            SOLDIER_SELECTABLE_SQUARE_SIDE,
        )
    }

    pub fn soldier_animation_type(&self, soldier: &Soldier) -> Box<dyn Sprite> {
        let animation_type = match soldier.behavior() {
            Behavior::Idle => SoldierAnimationType::Idle,
            Behavior::MoveTo(_) => SoldierAnimationType::Walking,
            Behavior::MoveFastTo(_) => SoldierAnimationType::Walking,
            Behavior::SneakTo(_) => SoldierAnimationType::Crawling,
            Behavior::Defend(_) => SoldierAnimationType::LyingDown,
            Behavior::Hide(_) => SoldierAnimationType::LyingDown,
            Behavior::DriveTo(_) => SoldierAnimationType::Idle,
            Behavior::RotateTo(_) => SoldierAnimationType::Idle,
            // TODO : Different animation according to death type
            Behavior::Dead => SoldierAnimationType::DeadWithSideBlood,
            Behavior::Unconscious => SoldierAnimationType::LyingDown,
            Behavior::SuppressFire(_) => SoldierAnimationType::LyingDown,
            Behavior::EngageSoldier(_) => SoldierAnimationType::LyingDown,
        };
        Box::new(animation_type)
    }
}
