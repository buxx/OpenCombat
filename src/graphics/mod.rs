use ggez::{
    graphics::{self, FilterMode},
    Context, GameResult,
};

use crate::{config, types::*};

pub struct Graphics {
    sprites_batch: graphics::spritebatch::SpriteBatch,
}

impl Graphics {
    pub fn new(ctx: &mut Context) -> GameResult<Graphics> {
        let mut sprites = graphics::Image::new(ctx, "/sprites.png")?;
        sprites.set_filter(FilterMode::Nearest); // because pixel art
        let sprites_batch = graphics::spritebatch::SpriteBatch::new(sprites);

        Ok(Graphics { sprites_batch })
    }

    pub fn extend(&mut self, sprites: Vec<graphics::DrawParam>) {
        for sprite in sprites {
            self.sprites_batch.add(sprite);
        }
    }

    pub fn entity_sprites(&self, _entity: &ThreadSafeEntity) -> Vec<graphics::DrawParam> {
        // TODO depending of a lot of things like entity type, physical behavior, etc
        let relative_start_x = 0. / config::SCENE_ITEMS_SPRITE_SHEET_WIDTH;
        let relative_start_y = 0. / config::SCENE_ITEMS_SPRITE_SHEET_HEIGHT;
        let relative_tile_width = 12. / config::SCENE_ITEMS_SPRITE_SHEET_WIDTH;
        let relative_tile_height = 12. / config::SCENE_ITEMS_SPRITE_SHEET_HEIGHT;
        vec![graphics::DrawParam::new().src(graphics::Rect::new(
            relative_start_x,
            relative_start_y,
            relative_tile_width,
            relative_tile_height,
        ))]
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        window_draw_param: graphics::DrawParam,
    ) -> GameResult {
        graphics::draw(ctx, &self.sprites_batch, window_draw_param)?;
        Ok(())
    }

    pub fn clear(&mut self, ctx: &mut Context) {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        self.sprites_batch.clear();
    }
}
