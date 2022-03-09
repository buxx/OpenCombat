use ggez::{
    graphics::{self, spritebatch::SpriteBatch, FilterMode, Image},
    Context, GameResult,
};

use crate::{config, map::Map, types::*};

mod map;

const SPRITES_FILE_PATH: &'static str = "/sprites.png";

pub struct Graphics {
    // Soldier, vehicle, explosions, etc sprite batch
    sprites_batch: SpriteBatch,
    // Map background sprite batch
    map_background_batch: SpriteBatch,
    // Map decor sprite batches
    map_decor_batches: Vec<SpriteBatch>,
}

impl Graphics {
    pub fn new(ctx: &mut Context, map: &Map) -> GameResult<Graphics> {
        let sprites_batch = get_sprites_batch(ctx)?;
        let map_background_batch = map::get_map_background_batch(ctx, map)?;
        let map_decor_batches = map::get_map_decor_batch(ctx, map)?;

        Ok(Graphics {
            sprites_batch,
            map_background_batch,
            map_decor_batches,
        })
    }

    pub fn append_sprites_batch(&mut self, sprite: graphics::DrawParam) {
        self.sprites_batch.add(sprite);
    }

    pub fn entity_sprites(&self, entity: &ThreadSafeEntity) -> Vec<graphics::DrawParam> {
        // TODO depending of a lot of things like entity type, physical behavior, etc
        let relative_start_x = 0. / config::SCENE_ITEMS_SPRITE_SHEET_WIDTH;
        let relative_start_y = 0. / config::SCENE_ITEMS_SPRITE_SHEET_HEIGHT;
        let relative_tile_width = 12. / config::SCENE_ITEMS_SPRITE_SHEET_WIDTH;
        let relative_tile_height = 12. / config::SCENE_ITEMS_SPRITE_SHEET_HEIGHT;
        vec![graphics::DrawParam::new()
            .src(graphics::Rect::new(
                relative_start_x,
                relative_start_y,
                relative_tile_width,
                relative_tile_height,
            ))
            .rotation(entity.get_looking_direction().0)
            .offset(Offset::new(0.5, 0.5).to_vec2())]
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        draw_decor: bool,
        window_draw_param: graphics::DrawParam,
    ) -> GameResult {
        // Map background sprites
        graphics::draw(ctx, &self.map_background_batch, window_draw_param)?;

        // Entities, explosions, etc. sprites
        graphics::draw(ctx, &self.sprites_batch, window_draw_param)?;

        // Draw decor like Trees
        if draw_decor {
            for decor_batch in self.map_decor_batches.iter() {
                graphics::draw(ctx, decor_batch, window_draw_param)?;
            }
        }

        Ok(())
    }

    pub fn clear(&mut self, ctx: &mut Context) {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        self.sprites_batch.clear();
    }
}

pub fn get_sprites_batch(ctx: &mut Context) -> GameResult<SpriteBatch> {
    let mut sprites_image = Image::new(ctx, SPRITES_FILE_PATH)?;
    sprites_image.set_filter(FilterMode::Nearest); // because pixel art
    let sprites_batch = SpriteBatch::new(sprites_image);

    Ok(sprites_batch)
}
