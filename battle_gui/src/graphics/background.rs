use std::path::PathBuf;

use battle_core::{map::Map, types::ScenePoint};
use ggez::{
    graphics::{DrawParam, Image, InstanceArray, Rect},
    Context, GameError, GameResult,
};

use crate::utils::qualified::ToQualified;

use super::{batch::QualifiedBatch, map::ensure_map_dark_backgrounds, qualified::Zoom};

pub struct Background {
    sd: InstanceArray,
    hd: InstanceArray,
}

impl Background {
    pub fn new(sd: InstanceArray, hd: InstanceArray) -> Self {
        Self { sd, hd }
    }
}

impl QualifiedBatch<InstanceArray> for Background {
    fn hd(&self) -> &InstanceArray {
        &self.hd
    }

    fn sd(&self) -> &InstanceArray {
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
            self.hd.push(draw)
        } else {
            self.sd.push(draw)
        }
    }
}

pub struct BackgroundBuilder<'a> {
    ctx: &'a mut Context,
    map: &'a Map,
    dark: bool,
}

impl<'a> BackgroundBuilder<'a> {
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

    pub fn build(&self) -> GameResult<Background> {
        Ok(Background::new(
            self.build_for(&Zoom::default())?,
            self.build_for(&Zoom::hd())?,
        ))
    }

    fn background_image_path(&self) -> GameResult<PathBuf> {
        if self.dark {
            ensure_map_dark_backgrounds(self.map)
        } else {
            Ok(self.map.background_image_path().clone())
        }
    }

    fn build_for(&self, zoom: &Zoom) -> GameResult<InstanceArray> {
        let image_path = self
            .background_image_path()?
            .to_qualified(zoom)
            .map_err(|error| {
                GameError::ResourceLoadError(format!(
                    "Background image source qualification error : {}",
                    error
                ))
            })?;
        let map_background_image = Image::from_path(self.ctx, image_path)?;
        let mut map_background_batch = InstanceArray::new(self.ctx, map_background_image);

        // This batch will never change, add draw param once
        map_background_batch.push(
            DrawParam::new()
                .src(Rect::new(0.0, 0.0, 1.0, 1.0))
                .dest(ScenePoint::new(0.0, 0.0).to_vec2()),
        );

        Ok(map_background_batch)
    }
}
