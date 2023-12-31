use super::AssetsType;

use battle_core::game::weapon::WeaponSprite;
use ggez::{
    graphics::{Canvas, DrawParam, Image, InstanceArray},
    Context, GameError, GameResult,
};

use crate::utils::qualified::ToQualified;

use super::{batch::QualifiedBatch, qualified::Zoom};

pub struct Weapons {
    riffle: Weapon,
}

pub struct Weapon {
    sd: InstanceArray,
    hd: InstanceArray,
}

impl Weapons {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            riffle: WeaponBuilder::new(ctx, WeaponSprite::Riffle).build()?,
        })
    }

    pub fn extend(&mut self, zoom: &Zoom, type_: WeaponSprite, weapon_sprites: Vec<DrawParam>) {
        match type_ {
            WeaponSprite::Riffle => self.riffle.extend(zoom, weapon_sprites),
        };
    }

    pub fn clear(&mut self, zoom: &Zoom) {
        self.riffle.clear(zoom);
    }

    pub fn draw(&self, canvas: &mut Canvas, zoom: &Zoom, draw_param: DrawParam) {
        canvas.draw(self.riffle.drawable(zoom), draw_param);
    }
}

impl Weapon {
    pub fn new(sd: InstanceArray, hd: InstanceArray) -> Self {
        Self { sd, hd }
    }
}

impl QualifiedBatch<InstanceArray> for Weapon {
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

pub struct WeaponBuilder<'a> {
    ctx: &'a mut Context,
    type_: WeaponSprite,
}

impl<'a> WeaponBuilder<'a> {
    pub fn new(ctx: &'a mut Context, type_: WeaponSprite) -> Self {
        Self { ctx, type_ }
    }

    pub fn build(&self) -> GameResult<Weapon> {
        Ok(Weapon::new(
            self.build_for(&Zoom::default())?,
            self.build_for(&Zoom::hd())?,
        ))
    }

    fn build_for(&self, zoom: &Zoom) -> GameResult<InstanceArray> {
        let image_path = AssetsType::Weapon(self.type_.clone())
            .path()
            .to_qualified(zoom)
            .map_err(|error| {
                GameError::ResourceLoadError(format!(
                    "Weapon image source qualification error : {}",
                    error
                ))
            })?;
        let image = Image::from_path(self.ctx, image_path)?;
        let batch = InstanceArray::new(self.ctx, image);
        Ok(batch)
    }
}
