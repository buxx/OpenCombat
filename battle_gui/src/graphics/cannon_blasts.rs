use super::AssetsType;

use ggez::{
    graphics::{DrawParam, Image, InstanceArray},
    Context, GameError, GameResult,
};

use crate::utils::qualified::ToQualified;

use super::{batch::QualifiedBatch, qualified::Zoom};

pub struct CannonBlasts {
    sd: InstanceArray,
    hd: InstanceArray,
}

impl CannonBlasts {
    pub fn new(sd: InstanceArray, hd: InstanceArray) -> Self {
        Self { sd, hd }
    }
}

impl QualifiedBatch<InstanceArray> for CannonBlasts {
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

pub struct CannonBlastsBuilder<'a> {
    ctx: &'a mut Context,
}

impl<'a> CannonBlastsBuilder<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }

    pub fn build(&self) -> GameResult<CannonBlasts> {
        Ok(CannonBlasts::new(
            self.build_for(&Zoom::default())?,
            self.build_for(&Zoom::hd())?,
        ))
    }

    fn build_for(&self, zoom: &Zoom) -> GameResult<InstanceArray> {
        let image_path = AssetsType::CannonBlasts
            .path()
            .to_qualified(zoom)
            .map_err(|error| {
                GameError::ResourceLoadError(format!(
                    "CannonBlasts image source qualification error : {}",
                    error
                ))
            })?;
        let image = Image::from_path(self.ctx, image_path)?;
        let batch = InstanceArray::new(self.ctx, image);
        Ok(batch)
    }
}
