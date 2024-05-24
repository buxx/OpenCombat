use battle_core::{
    config::{SOLDIER_SELECTABLE_SQUARE_SIDE, SOLDIER_SELECTABLE_SQUARE_SIDE_HALF},
    entity::soldier::Soldier,
};
use ggez::graphics::Rect;
use keyframe::AnimationSequence;
use oc_core::{
    game::soldier::SoldierType,
    graphics::squad::{
        SOLDIER_REL_1_START_X, SOLDIER_REL_1_START_Y, SOLDIER_REL_BREN_START_X,
        SOLDIER_REL_BREN_START_Y, SOLDIER_REL_HEIGHT, SOLDIER_REL_MG34_START_X,
        SOLDIER_REL_MG34_START_Y, SOLDIER_REL_WIDTH,
    },
};

use super::{AssetsType, Graphics, TweenableRect};

use ggez::{
    graphics::{DrawParam, Image, InstanceArray},
    Context, GameError, GameResult,
};

use super::utils::IntoDrawParam;
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
                    error
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
}

pub struct SoldierAnimationSequence {
    soldier: AnimationSequence<TweenableRect>,
    weapon: Option<AnimationSequence<TweenableRect>>,
}

impl SoldierAnimationSequence {
    pub fn new(
        soldier: AnimationSequence<TweenableRect>,
        weapon: Option<AnimationSequence<TweenableRect>>,
    ) -> Self {
        Self { soldier, weapon }
    }

    pub fn soldier(&self) -> &AnimationSequence<TweenableRect> {
        &self.soldier
    }

    pub fn weapon(&self) -> Option<&AnimationSequence<TweenableRect>> {
        self.weapon.as_ref()
    }

    pub fn soldier_mut(&mut self) -> &mut AnimationSequence<TweenableRect> {
        &mut self.soldier
    }

    pub fn weapon_mut(&mut self) -> &mut Option<AnimationSequence<TweenableRect>> {
        &mut self.weapon
    }
}

impl IntoDrawParam for SoldierType {
    fn to_draw_param(&self) -> DrawParam {
        match self {
            SoldierType::Type1 => DrawParam::new().src(Rect::new(
                SOLDIER_REL_1_START_X,
                SOLDIER_REL_1_START_Y,
                SOLDIER_REL_WIDTH,
                SOLDIER_REL_HEIGHT,
            )),
            SoldierType::Bren => DrawParam::new().src(Rect::new(
                SOLDIER_REL_BREN_START_X,
                SOLDIER_REL_BREN_START_Y,
                SOLDIER_REL_WIDTH,
                SOLDIER_REL_HEIGHT,
            )),
            SoldierType::Mg34 => DrawParam::new().src(Rect::new(
                SOLDIER_REL_MG34_START_X,
                SOLDIER_REL_MG34_START_Y,
                SOLDIER_REL_WIDTH,
                SOLDIER_REL_HEIGHT,
            )),
        }
    }
}
