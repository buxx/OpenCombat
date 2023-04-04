use super::AssetsType;

use ggez::{
    graphics::{DrawParam, Image, InstanceArray},
    Context, GameError, GameResult,
};

use crate::utils::qualified::ToQualified;

use super::{batch::QualifiedBatch, qualified::Zoom};

pub struct Vehicles {
    sd: InstanceArray,
    hd: InstanceArray,
}

impl Vehicles {
    pub fn new(sd: InstanceArray, hd: InstanceArray) -> Self {
        Self { sd, hd }
    }
}

impl QualifiedBatch<InstanceArray> for Vehicles {
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

pub struct VehiclesBuilder<'a> {
    ctx: &'a mut Context,
}

impl<'a> VehiclesBuilder<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }

    pub fn build(&self) -> GameResult<Vehicles> {
        Ok(Vehicles::new(
            self.build_for(&Zoom::Standard)?,
            self.build_for(&Zoom::In)?,
        ))
    }

    fn build_for(&self, zoom: &Zoom) -> GameResult<InstanceArray> {
        let image_path = AssetsType::Vehicles
            .path()
            .to_qualified(zoom)
            .map_err(|error| {
                GameError::ResourceLoadError(format!(
                    "Vehicles image source qualification error : {}",
                    error.to_string()
                ))
            })?;
        let image = Image::from_path(self.ctx, image_path)?;
        let batch = InstanceArray::new(self.ctx, image);
        Ok(batch)
    }
}
