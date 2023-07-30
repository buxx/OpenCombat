use battle_core::{map::Map, types::ScenePoint};
use ggez::{
    graphics::{DrawParam, Image, InstanceArray, Rect},
    Context, GameError, GameResult,
};

use crate::utils::qualified::ToQualified;

use super::{batch::QualifiedBatch, qualified::Zoom};

pub struct Decors {
    sd: Vec<InstanceArray>,
    hd: Vec<InstanceArray>,
}

impl Decors {
    pub fn new(sd: Vec<InstanceArray>, hd: Vec<InstanceArray>) -> Self {
        Self { sd, hd }
    }
}

impl QualifiedBatch<Vec<InstanceArray>> for Decors {
    fn hd(&self) -> &Vec<InstanceArray> {
        &self.hd
    }

    fn sd(&self) -> &Vec<InstanceArray> {
        &self.sd
    }

    fn clear(&mut self, zoom: &Zoom) {
        if zoom.is_hd() {
            self.hd.clear()
        } else {
            self.sd.clear()
        }
    }

    fn push(&mut self, zoom: &Zoom, draw: DrawParam) {
        if zoom.is_hd() {
            self.hd.iter_mut().for_each(|a| a.push(draw))
        } else {
            self.sd.iter_mut().for_each(|a| a.push(draw))
        }
    }
}

pub struct DecorsBuilder<'a> {
    ctx: &'a mut Context,
    map: &'a Map,
    dark: bool,
}

impl<'a> DecorsBuilder<'a> {
    pub fn new(ctx: &'a mut Context, map: &'a Map) -> Self {
        Self {
            ctx,
            map,
            dark: false,
        }
    }

    pub fn dark(mut self, value: bool) -> Self {
        self.dark = value;
        self
    }

    pub fn build(&self) -> GameResult<Decors> {
        Ok(Decors::new(
            self.build_for(&Zoom::default())?,
            self.build_for(&Zoom::hd())?,
        ))
    }

    fn build_for(&self, zoom: &Zoom) -> GameResult<Vec<InstanceArray>> {
        let mut map_decor_batches = vec![];
        for image_path in self.map.decor().image_paths() {
            let image_path = image_path.to_qualified(zoom).map_err(|error| {
                GameError::ResourceLoadError(format!(
                    "Decor image source qualification error : {}",
                    error.to_string()
                ))
            })?;
            let decor_image_path = if self.dark {
                image_path.to_dark(self.map.name()).map_err(|error| {
                    GameError::ResourceLoadError(format!(
                        "Decor image source dark version error : {}",
                        error.to_string()
                    ))
                })?
            } else {
                image_path
            };
            let decor_image = Image::from_path(self.ctx, decor_image_path)?;
            let batch = InstanceArray::new(self.ctx, decor_image);
            map_decor_batches.push(batch);
        }

        for tile in self.map.decor().tiles() {
            let decor_batch = map_decor_batches
                .get_mut(tile.tileset_i)
                .expect("Batch must be here");

            // Tiled draw from bottom left but we draw from top left, so compute a decal
            let dest_decal = tile.tile_height as f32 - self.map.tile_height() as f32;
            let src_x = tile.tile_x as f32 * tile.relative_tile_width;
            let src_y = tile.tile_y as f32 * tile.relative_tile_height;
            // Destination computation refer to terrain grid (map.terrain.tileset)
            let dest_x = tile.x as f32 * self.map.tile_width() as f32;
            let dest_y = (tile.y as f32 * self.map.tile_height() as f32) - dest_decal;
            let dest = ScenePoint::new(dest_x, dest_y).to_vec2() * zoom.factor();

            decor_batch.push(
                DrawParam::new()
                    .src(Rect::new(
                        src_x,
                        src_y,
                        tile.relative_tile_width,
                        tile.relative_tile_height,
                    ))
                    .dest(dest),
            );
        }

        Ok(map_decor_batches)
    }
}
