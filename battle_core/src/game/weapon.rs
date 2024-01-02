use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::{
    audio::Sound,
    graphics::{cannon_blast::CannonBlastAnimationType, Sprite},
};

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Ammunition {
    x762x54R,
    x792x57,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Magazine {
    MosinNagant(usize),
    Mauser(usize),
}

impl Magazine {
    pub fn name(&self) -> &str {
        match self {
            Magazine::MosinNagant(_) => "Mosin Nagant",
            Magazine::Mauser(_) => "Mauser",
        }
    }

    pub fn full(magazine: Self) -> Self {
        match magazine {
            Magazine::MosinNagant(_) => Magazine::MosinNagant(5),
            Magazine::Mauser(_) => Magazine::Mauser(5),
        }
    }

    pub fn ammunition(&self) -> Ammunition {
        match self {
            Magazine::MosinNagant(_) => Ammunition::x762x54R,
            Magazine::Mauser(_) => Ammunition::x792x57,
        }
    }

    pub fn filled(&self) -> bool {
        match self {
            Magazine::MosinNagant(fill) => *fill > 0,
            Magazine::Mauser(fill) => *fill > 0,
        }
    }

    fn remove_one(&mut self) {
        match self {
            Magazine::MosinNagant(fill) => {
                if *fill > 0 {
                    *fill -= 1;
                }
            }
            Magazine::Mauser(fill) => {
                if *fill > 0 {
                    *fill -= 1;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeaponSprite {
    Riffle,
}

impl WeaponSprite {
    pub fn prefix(&self) -> &str {
        match self {
            WeaponSprite::Riffle => "/weapon_riffle",
        }
    }

    pub fn sprite(&self) -> Box<dyn Sprite> {
        let animation_type = match self {
            WeaponSprite::Riffle => CannonBlastAnimationType::RiffleOneShotOnLying,
        };
        Box::new(animation_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GunFireSoundType {
    MosinNagant,
    MauserRiffle,
}

impl GunFireSoundType {
    pub fn fire_sounds(&self) -> Vec<Sound> {
        let pick_from = match self {
            GunFireSoundType::MosinNagant => vec![
                Sound::MosinNagantFire1,
                Sound::MosinNagantFire2,
                Sound::MosinNagantFire3,
                Sound::MosinNagantFire4,
                Sound::MosinNagantFire5,
            ],
            GunFireSoundType::MauserRiffle => vec![
                Sound::MauserRiffleFire1,
                Sound::MauserRiffleFire2,
                Sound::MauserRiffleFire3,
            ],
        };
        let sound = *pick_from
            .choose(&mut rand::thread_rng())
            .expect("Must one be chosen");

        vec![sound]
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Weapon {
    // ready bullet, filled magazine
    MosinNagantM1924(bool, Option<Magazine>),
    MauserG41(bool, Option<Magazine>),
}

impl Weapon {
    pub fn name(&self) -> &str {
        match self {
            Weapon::MosinNagantM1924(_, _) => "Mosin Nagant M1924",
            Weapon::MauserG41(_, _) => "Mauser G41",
        }
    }

    pub fn gun_fire_sound_type(&self) -> GunFireSoundType {
        match self {
            Weapon::MosinNagantM1924(_, _) => GunFireSoundType::MosinNagant,
            Weapon::MauserG41(_, _) => GunFireSoundType::MauserRiffle,
        }
    }

    pub fn reload_sounds(&self) -> Vec<Sound> {
        let pick_from = match self {
            Weapon::MosinNagantM1924(_, _) => vec![
                Sound::MosinNagantReload1,
                Sound::MosinNagantReload2,
                Sound::MosinNagantReload3,
                Sound::MosinNagantReload4,
            ],
            Weapon::MauserG41(_, _) => vec![Sound::MauserRiffleReload1, Sound::MauserRiffleReload2],
        };
        let sound = *pick_from
            .choose(&mut rand::thread_rng())
            .expect("Must one be chosen");

        vec![sound]
    }

    pub fn magazine(&self) -> &Option<Magazine> {
        match self {
            Weapon::MosinNagantM1924(_, magazine) => magazine,
            Weapon::MauserG41(_, magazine) => magazine,
        }
    }

    pub fn accepted_magazine(&self, magazine: &Magazine) -> bool {
        match self {
            Weapon::MosinNagantM1924(_, _) => matches!(magazine, Magazine::MosinNagant(_)),
            Weapon::MauserG41(_, _) => matches!(magazine, Magazine::Mauser(_)),
        }
    }

    pub fn ammunition(&self) -> Ammunition {
        if let Some(magazine) = self.magazine() {
            return magazine.ammunition();
        }

        // Default value
        match self {
            Weapon::MosinNagantM1924(_, _) => Ammunition::x762x54R,
            Weapon::MauserG41(_, _) => Ammunition::x792x57,
        }
    }

    pub fn can_fire(&self) -> bool {
        match self {
            Weapon::MosinNagantM1924(ammunition, _) => *ammunition,
            Weapon::MauserG41(ammunition, _) => *ammunition,
        }
    }

    pub fn can_reload(&self) -> bool {
        match self {
            Weapon::MosinNagantM1924(_, magazine) => {
                if let Some(magazine) = magazine {
                    return magazine.filled();
                }
            }
            Weapon::MauserG41(_, magazine) => {
                if let Some(magazine) = magazine {
                    return magazine.filled();
                }
            }
        }

        false
    }

    pub fn reload(&mut self) {
        match self {
            Weapon::MosinNagantM1924(ready_bullet, magazine)
            | Weapon::MauserG41(ready_bullet, magazine) => {
                if !*ready_bullet {
                    if let Some(magazine_) = magazine {
                        if magazine_.filled() {
                            magazine_.remove_one();
                            *ready_bullet = true;
                        }

                        if !magazine_.filled() {
                            *magazine = None;
                        }
                    }
                }
            }
        }
    }

    pub fn shot(&mut self) {
        match self {
            Weapon::MosinNagantM1924(ready_bullet, _) => *ready_bullet = false,
            Weapon::MauserG41(ready_bullet, _) => *ready_bullet = false,
        }
    }

    pub fn set_magazine(&mut self, new_magazine: Magazine) {
        match self {
            Weapon::MosinNagantM1924(_, magazine) => *magazine = Some(new_magazine),
            Weapon::MauserG41(_, magazine) => *magazine = Some(new_magazine),
        }
    }

    pub fn ok_count_magazines(&self) -> usize {
        match self {
            Weapon::MosinNagantM1924(_, _) => 5,
            Weapon::MauserG41(_, _) => 5,
        }
    }

    pub fn sprite_type(&self) -> WeaponSprite {
        match self {
            Weapon::MosinNagantM1924(_, _) | Weapon::MauserG41(_, _) => WeaponSprite::Riffle,
        }
    }
}
