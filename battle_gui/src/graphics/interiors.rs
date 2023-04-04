use battle_core::{map::Map, types::ScenePoint};
use ggez::{
    graphics::{DrawParam, Image, InstanceArray, Rect},
    Context, GameError, GameResult,
};

use crate::utils::qualified::ToQualified;

use super::{batch::QualifiedBatch, qualified::Zoom};

pub struct Interiors {
    sd: InstanceArray,
    hd: InstanceArray,
}

impl Interiors {
    pub fn new(sd: InstanceArray, hd: InstanceArray) -> Self {
        Self { sd, hd }
    }
}

impl QualifiedBatch<InstanceArray> for Interiors {
    fn hd(&self) -> &InstanceArray {
        &self.hd
    }

    fn sd(&self) -> &InstanceArray {
        &self.sd
    }

    fn clear(&mut self, zoom: &Zoom) {
        match zoom {
            Zoom::In => self.hd.clear(),
            _ => self.sd.clear(),
        }
    }

    fn push(&mut self, zoom: &Zoom, draw: DrawParam) {
        match zoom {
            Zoom::In => self.hd.push(draw),
            _ => self.sd.push(draw),
        }
    }
}

pub struct InteriorsBuilder<'a> {
    ctx: &'a mut Context,
    map: &'a Map,
}

impl<'a> InteriorsBuilder<'a> {
    pub fn new(ctx: &'a mut Context, map: &'a Map) -> Self {
        Self { ctx, map }
    }

    pub fn build(&self) -> GameResult<Interiors> {
        Ok(Interiors::new(
            self.build_for(&Zoom::Standard)?,
            self.build_for(&Zoom::In)?,
        ))
    }

    fn build_for(&self, zoom: &Zoom) -> GameResult<InstanceArray> {
        let image_path = self
            .map
            .interiors_image_path()
            .to_qualified(zoom)
            .map_err(|error| {
                GameError::ResourceLoadError(format!(
                    "Interiors image source qualification error : {}",
                    error.to_string()
                ))
            })?;
        let map_interiors_image = Image::from_path(self.ctx, image_path)?;
        let mut map_interiors_batch = InstanceArray::new(self.ctx, map_interiors_image);

        // This batch will never change, add draw param once
        map_interiors_batch.push(
            DrawParam::new()
                .src(Rect::new(0.0, 0.0, 1.0, 1.0))
                .dest(ScenePoint::new(0.0, 0.0).to_vec2()),
        );

        Ok(map_interiors_batch)
    }
}
