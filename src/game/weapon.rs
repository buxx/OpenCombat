use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::audio::Sound;

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Ammunition {
    x762x54R,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Magazine {
    MosinNagant(usize),
}

impl Magazine {
    pub fn full(magazine: Self) -> Self {
        match magazine {
            Magazine::MosinNagant(_) => Magazine::MosinNagant(5),
        }
    }

    pub fn ammunition(&self) -> Ammunition {
        match self {
            Magazine::MosinNagant(_) => Ammunition::x762x54R,
        }
    }

    pub fn filled(&self) -> bool {
        match self {
            Magazine::MosinNagant(fill) => *fill > 0,
        }
    }

    fn remove_one(&mut self) {
        match self {
            Magazine::MosinNagant(fill) => {
                if *fill > 0 {
                    *fill = *fill - 1;
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Weapon {
    // ready bullet, filled magazine
    MosinNagantM1924(bool, Option<Magazine>),
}

impl Weapon {
    pub fn fire_sounds(&self) -> Vec<Sound> {
        let pick_from = match self {
            Weapon::MosinNagantM1924(_, _) => vec![
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

    pub fn reload_sounds(&self) -> Vec<Sound> {
        let pick_from = match self {
            Weapon::MosinNagantM1924(_, _) => vec![
                Sound::MosinNagantReload1,
                Sound::MosinNagantReload2,
                Sound::MosinNagantReload3,
                Sound::MosinNagantReload4,
            ],
        };
        let sound = *pick_from
            .choose(&mut rand::thread_rng())
            .expect("Must one be chosen");

        vec![sound]
    }

    pub fn magazine(&self) -> &Option<Magazine> {
        match self {
            Weapon::MosinNagantM1924(_, magazine) => magazine,
        }
    }

    pub fn accepted_magazine(&self, magazine: &Magazine) -> bool {
        match magazine {
            Magazine::MosinNagant(_) => true,
        }
    }

    pub fn ammunition(&self) -> Ammunition {
        if let Some(magazine) = self.magazine() {
            return magazine.ammunition();
        }

        // Default value
        match self {
            Weapon::MosinNagantM1924(_, _) => Ammunition::x762x54R,
        }
    }

    pub fn can_fire(&self) -> bool {
        match self {
            Weapon::MosinNagantM1924(ammunition, _) => *ammunition,
        }
    }

    pub fn can_reload_and_shoot(&self) -> bool {
        match self {
            Weapon::MosinNagantM1924(_, magazine) => {
                if let Some(magazine) = magazine {
                    return magazine.filled();
                }
            }
        }

        false
    }

    pub fn reload(&mut self) {
        match self {
            Weapon::MosinNagantM1924(ready_bullet, magazine) => {
                if !*ready_bullet {
                    if let Some(magazine) = magazine {
                        if magazine.filled() {
                            magazine.remove_one();
                            *ready_bullet = true;
                        }
                    }
                }
            }
        }
    }

    pub fn shot(&mut self) {
        match self {
            Weapon::MosinNagantM1924(ready_bullet, _) => *ready_bullet = false,
        }
    }
}
