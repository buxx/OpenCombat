use ggez::{
    graphics::{self, spritebatch::SpriteBatch, FilterMode, Image, MeshBuilder},
    Context, GameResult,
};

use crate::{config, map::Map, types::*, ui::menu::squad_menu_sprite_info};

mod map;

const SPRITES_FILE_PATH: &'static str = "/sprites.png";
const UI_FILE_PATH: &'static str = "/ui.png";

pub struct Graphics {
    // Soldier, vehicle, explosions, etc sprite batch
    sprites_batch: SpriteBatch,
    // Squad menu, etc
    ui_batch: SpriteBatch,
    // Map background sprite batch
    map_background_batch: SpriteBatch,
    // Map decor sprite batches
    map_decor_batches: Vec<SpriteBatch>,
}

impl Graphics {
    pub fn new(ctx: &mut Context, map: &Map) -> GameResult<Graphics> {
        let sprites_batch = create_sprites_batch(ctx)?;
        let ui_batch = create_ui_batch(ctx)?;
        let map_background_batch = map::get_map_background_batch(ctx, map)?;
        let map_decor_batches = map::get_map_decor_batch(ctx, map)?;

        Ok(Graphics {
            sprites_batch,
            ui_batch,
            map_background_batch,
            map_decor_batches,
        })
    }

    pub fn append_sprites_batch(&mut self, sprite: graphics::DrawParam) {
        self.sprites_batch.add(sprite);
    }

    pub fn append_ui_batch(&mut self, sprite: graphics::DrawParam) {
        self.ui_batch.add(sprite);
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

    pub fn squad_menu_sprites(
        &self,
        to_point: WindowPoint,
        cursor_point: WindowPoint,
        _squad_id: SquadUuid,
    ) -> Vec<graphics::DrawParam> {
        squad_menu_sprite_info().as_draw_params(&to_point, &cursor_point)
    }

    pub fn draw_scene(
        &mut self,
        ctx: &mut Context,
        draw_decor: bool,
        draw_param: graphics::DrawParam,
    ) -> GameResult {
        // Map background sprites
        graphics::draw(ctx, &self.map_background_batch, draw_param)?;

        // Entities, explosions, etc. sprites
        graphics::draw(ctx, &self.sprites_batch, draw_param)?;

        // Draw decor like Trees
        if draw_decor {
            for decor_batch in self.map_decor_batches.iter() {
                graphics::draw(ctx, decor_batch, draw_param)?;
            }
        }

        Ok(())
    }

    pub fn draw_ui(
        &mut self,
        ctx: &mut Context,
        draw_param: graphics::DrawParam,
        mesh_builder: MeshBuilder,
    ) -> GameResult {
        // Different meshes
        if let Ok(mesh) = mesh_builder.build(ctx) {
            graphics::draw(ctx, &mesh, draw_param)?;
        };

        // Squad menu, etc
        graphics::draw(ctx, &self.ui_batch, draw_param)?;

        Ok(())
    }

    pub fn clear(&mut self, ctx: &mut Context) {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        self.sprites_batch.clear();
        self.ui_batch.clear();
    }
}

pub fn create_sprites_batch(ctx: &mut Context) -> GameResult<SpriteBatch> {
    let mut sprites_image = Image::new(ctx, SPRITES_FILE_PATH)?;
    sprites_image.set_filter(FilterMode::Nearest); // because pixel art
    let sprites_batch = SpriteBatch::new(sprites_image);

    Ok(sprites_batch)
}

pub fn create_ui_batch(ctx: &mut Context) -> GameResult<SpriteBatch> {
    let mut ui_image = Image::new(ctx, UI_FILE_PATH)?;
    ui_image.set_filter(FilterMode::Nearest); // because pixel art
    let ui_batch = SpriteBatch::new(ui_image);

    Ok(ui_batch)
}
