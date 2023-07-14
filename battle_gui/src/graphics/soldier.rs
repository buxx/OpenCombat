use battle_core::{
    behavior::Behavior,
    config::{SOLDIER_SELECTABLE_SQUARE_SIDE, SOLDIER_SELECTABLE_SQUARE_SIDE_HALF},
    entity::soldier::Soldier,
    graphics::{soldier::SoldierAnimationType, Sprite},
};
use ggez::graphics::Rect;

use super::{AssetsType, Graphics};

use ggez::{
    graphics::{DrawParam, Image, InstanceArray},
    Context, GameError, GameResult,
};

use crate::utils::qualified::ToQualified;

use super::{batch::QualifiedBatch, qualified::Zoom};

pub struct Soldiers {
    sd: InstanceArray,
    hd: InstanceArray,
}

impl Soldiers {
    pub fn new(sd: InstanceArray, hd: InstanceArray) -> Self {
        Self { sd, hd }
    }
}

impl QualifiedBatch<InstanceArray> for Soldiers {
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

pub struct SoldiersBuilder<'a> {
    ctx: &'a mut Context,
}

impl<'a> SoldiersBuilder<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }

    pub fn build(&self) -> GameResult<Soldiers> {
        Ok(Soldiers::new(
            self.build_for(&Zoom::default())?,
            self.build_for(&Zoom::hd())?,
        ))
    }

    fn build_for(&self, zoom: &Zoom) -> GameResult<InstanceArray> {
        let image_path = AssetsType::Soldiers
            .path()
            .to_qualified(zoom)
            .map_err(|error| {
                GameError::ResourceLoadError(format!(
                    "Soldiers image source qualification error : {}",
                    error.to_string()
                ))
            })?;
        let image = Image::from_path(self.ctx, image_path)?;
        let batch = InstanceArray::new(self.ctx, image);
        Ok(batch)
    }
}

impl Graphics {
    pub fn soldier_selection_rect(&self, soldier: &Soldier) -> Rect {
        Rect::new(
            soldier.world_point().x - SOLDIER_SELECTABLE_SQUARE_SIDE_HALF,
            soldier.world_point().y - SOLDIER_SELECTABLE_SQUARE_SIDE_HALF,
            SOLDIER_SELECTABLE_SQUARE_SIDE,
            SOLDIER_SELECTABLE_SQUARE_SIDE,
        )
    }

    pub fn soldier_animation_type(&self, soldier: &Soldier) -> Box<dyn Sprite> {
        let animation_type = match soldier.behavior() {
            Behavior::Idle => SoldierAnimationType::Idle,
            Behavior::MoveTo(_) => SoldierAnimationType::Walking,
            Behavior::MoveFastTo(_) => SoldierAnimationType::Walking,
            Behavior::SneakTo(_) => SoldierAnimationType::Crawling,
            Behavior::Defend(_) => SoldierAnimationType::LyingDown,
            Behavior::Hide(_) => SoldierAnimationType::LyingDown,
            Behavior::DriveTo(_) => SoldierAnimationType::Idle,
            Behavior::RotateTo(_) => SoldierAnimationType::Idle,
            // TODO : Different animation according to death type
            Behavior::Dead => SoldierAnimationType::DeadWithSideBlood,
            Behavior::Unconscious => SoldierAnimationType::LyingDown,
            Behavior::SuppressFire(_) => SoldierAnimationType::LyingDown,
            Behavior::EngageSoldier(_) => SoldierAnimationType::LyingDown,
        };
        Box::new(animation_type)
    }
}
