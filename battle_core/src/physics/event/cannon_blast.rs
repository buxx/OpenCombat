use crate::{
    config::TARGET_FPS,
    game::weapon::WeaponSprite,
    graphics::soldier::SoldierAnimationType,
    types::{Angle, WorldPoint},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CannonBlast {
    start: u64,
    end: u64,
    /// Soldier reference point
    point: WorldPoint,
    angle: Angle,
    weapon_sprite_type: WeaponSprite,
    soldier_animation_type: SoldierAnimationType,
}

impl CannonBlast {
    pub fn new(
        point: WorldPoint,
        angle: Angle,
        type_: WeaponSprite,
        soldier_animation_type: SoldierAnimationType,
    ) -> Self {
        Self {
            start: 0,
            end: 0,
            point,
            angle,
            weapon_sprite_type: type_,
            soldier_animation_type,
        }
    }

    pub fn init(&mut self, start_frame_i: u64) {
        self.start = start_frame_i;
        self.end = start_frame_i
            + (self.weapon_sprite_type.sprite().duration() * TARGET_FPS as f32) as u64;
    }

    pub fn point(&self) -> &WorldPoint {
        &self.point
    }

    pub fn angle(&self) -> &Angle {
        &self.angle
    }

    pub fn type_(&self) -> &WeaponSprite {
        &self.weapon_sprite_type
    }

    pub fn finished(&self, frame_i: u64) -> bool {
        frame_i >= self.end
    }

    pub fn effective(&self, frame_i: u64) -> bool {
        self.start == frame_i
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        self.end
    }

    pub fn weapon_sprite_type(&self) -> &WeaponSprite {
        &self.weapon_sprite_type
    }

    pub fn soldier_animation_type(&self) -> &SoldierAnimationType {
        &self.soldier_animation_type
    }
}
