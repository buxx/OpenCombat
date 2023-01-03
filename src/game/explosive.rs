use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::{
    audio::Sound,
    graphics::{animation::Sprite, explosion::ExplosionAnimationType},
    types::BlastPower,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    FA19241927,
}

impl Type {
    pub fn sounds(&self) -> Vec<Sound> {
        let pick_from = match self {
            Type::FA19241927 => vec![Sound::CannonFire1],
        };
        let sound = *pick_from
            .choose(&mut rand::thread_rng())
            .expect("Must one be chosen");

        vec![sound]
    }

    pub fn sprite(&self) -> Box<dyn Sprite> {
        let animation_type = match self {
            Type::FA19241927 => ExplosionAnimationType::Explosion1,
        };
        Box::new(animation_type)
    }

    pub fn blast(&self) -> BlastPower {
        BlastPower(match self {
            Type::FA19241927 => 50,
        })
    }
}
