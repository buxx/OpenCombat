use ggez::{graphics, GameResult};
use glam::Vec2;

use crate::types::*;

use super::Engine;

impl Engine {
    // TODO : don't generate sprites of non visible entities (hidden enemy, outside screen, etc)
    pub fn entity_sprites(
        &self,
        entity: &ThreadSafeEntity,
    ) -> GameResult<Vec<graphics::DrawParam>> {
        let mut sprites = vec![];

        for sprite in self.graphics.entity_sprites(entity) {
            let draw_to: Vec2 = entity.get_world_point().into();
            sprites.push(sprite.dest(draw_to));
        }

        Ok(sprites)
    }
}
