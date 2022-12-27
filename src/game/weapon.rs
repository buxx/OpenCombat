use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::audio::Sound;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Weapon {
    MosinNagantM1924,
}

impl Weapon {
    pub fn fire_sounds(&self) -> Vec<Sound> {
        let pick_from = match self {
            Weapon::MosinNagantM1924 => vec![
                Sound::MosinNagantFire1,
                Sound::MosinNagantFire2,
                Sound::MosinNagantFire3,
                Sound::MosinNagantFire4,
                Sound::MosinNagantFire5,
            ],
        };
        let sound = *pick_from
            .choose(&mut rand::thread_rng())
            .expect("Must one be chosen");

        vec![sound]
    }
}
