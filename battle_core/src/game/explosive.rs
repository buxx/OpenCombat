use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    audio::Sound,
    graphics::{explosion::ExplosionAnimationType, Sprite},
    types::Distance,
};

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, PartialEq, Eq, Hash)]
pub enum ExplosiveType {
    FA19241927,
}

impl ExplosiveType {
    pub fn sounds(&self) -> Vec<Sound> {
        let pick_from = match self {
            ExplosiveType::FA19241927 => vec![Sound::CannonFire1],
        };
        let sound = *pick_from
            .choose(&mut rand::thread_rng())
            .expect("Must one be chosen");

        vec![sound]
    }

    pub fn sprite(&self) -> Box<dyn Sprite> {
        let animation_type = match self {
            ExplosiveType::FA19241927 => ExplosionAnimationType::Explosion1,
        };
        Box::new(animation_type)
    }

    pub fn direct_death_perimeter(&self) -> Distance {
        match self {
            ExplosiveType::FA19241927 => Distance::from_meters(1),
        }
    }

    pub fn regressive_death_perimeter(&self) -> Distance {
        match self {
            ExplosiveType::FA19241927 => Distance::from_meters(3),
        }
    }

    pub fn regressive_injured_perimeter(&self) -> Distance {
        match self {
            ExplosiveType::FA19241927 => Distance::from_meters(6),
        }
    }
}
