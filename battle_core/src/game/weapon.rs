use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::{
    audio::Sound,
    config::TARGET_FPS,
    graphics::{cannon_blast::CannonBlastAnimationType, Sprite},
};

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Ammunition {
    x762x54R,
    x792x57,
    x303British,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Magazine {
    MosinNagant(usize),
    Mauser(usize),
    BrenCurved30(usize),
}

impl Magazine {
    pub fn name(&self) -> &str {
        match self {
            Magazine::MosinNagant(_) => "Mosin Nagant",
            Magazine::Mauser(_) => "Mauser",
            Magazine::BrenCurved30(_) => "Bren curved",
        }
    }

    pub fn full(magazine: Self) -> Self {
        match magazine {
            Magazine::MosinNagant(_) => Magazine::MosinNagant(5),
            Magazine::Mauser(_) => Magazine::Mauser(5),
            Magazine::BrenCurved30(_) => Magazine::BrenCurved30(30),
        }
    }

    pub fn ammunition(&self) -> Ammunition {
        match self {
            Magazine::MosinNagant(_) => Ammunition::x762x54R,
            Magazine::Mauser(_) => Ammunition::x792x57,
            Magazine::BrenCurved30(_) => Ammunition::x303British,
        }
    }

    pub fn filled(&self) -> bool {
        match self {
            Magazine::MosinNagant(fill) => *fill > 0,
            Magazine::Mauser(fill) => *fill > 0,
            Magazine::BrenCurved30(fill) => *fill > 0,
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
            Magazine::BrenCurved30(fill) => {
                if *fill > 0 {
                    *fill -= 1;
                }
            }
        }
    }

    fn remove(&mut self, count: usize) {
        match self {
            Magazine::BrenCurved30(fill) => {
                if *fill < count {
                    eprintln!(
                        "Tried to remove {} bullet from magazine with {} ammo ",
                        count, fill
                    )
                }

                if *fill >= count {
                    *fill -= count;
                } else {
                    *fill = 0
                }
            }
            _ => {
                if count > 1 {
                    eprintln!(
                        "Tried to remove {} bullet from {} magazine ",
                        count,
                        self.name()
                    )
                } else {
                    self.remove_one()
                }
            }
        }
    }

    pub fn count(&self) -> usize {
        match self {
            Magazine::MosinNagant(count)
            | Magazine::Mauser(count)
            | Magazine::BrenCurved30(count) => *count,
        }
    }

    fn very_long_shot(&self) -> Shot {
        match match self {
            Magazine::MosinNagant(_) => Ok(Shot::x1),
            Magazine::Mauser(_) => Ok(Shot::x1),
            Magazine::BrenCurved30(count) => Shot::try_from(*count.min(&16)),
        } {
            Ok(shot) => shot,
            Err(err) => {
                eprintln!(
                    "Tried to find very_long_shot on {} but not shot for value {}",
                    self.name(),
                    err.0
                );
                Shot::x1
            }
        }
    }

    fn long_shot(&self) -> Shot {
        match match self {
            Magazine::MosinNagant(_) => Ok(Shot::x1),
            Magazine::Mauser(_) => Ok(Shot::x1),
            Magazine::BrenCurved30(count) => Shot::try_from(*count.min(&10)),
        } {
            Ok(shot) => shot,
            Err(err) => {
                eprintln!(
                    "Tried to find long_shot on {} but not shot for value {}",
                    self.name(),
                    err.0
                );
                Shot::x1
            }
        }
    }

    fn medium_shot(&self) -> Shot {
        match match self {
            Magazine::MosinNagant(_) => Ok(Shot::x1),
            Magazine::Mauser(_) => Ok(Shot::x1),
            Magazine::BrenCurved30(count) => Shot::try_from(*count.min(&5)),
        } {
            Ok(shot) => shot,
            Err(err) => {
                eprintln!(
                    "Tried to find medium_shot on {} but not shot for value {}",
                    self.name(),
                    err.0
                );
                Shot::x1
            }
        }
    }

    fn short_shot(&self) -> Shot {
        match match self {
            Magazine::MosinNagant(_) => Ok(Shot::x1),
            Magazine::Mauser(_) => Ok(Shot::x1),
            Magazine::BrenCurved30(count) => Shot::try_from(*count.min(&3)),
        } {
            Ok(shot) => shot,
            Err(err) => {
                eprintln!(
                    "Tried to find short_shot on {} but not shot for value {}",
                    self.name(),
                    err.0
                );
                Shot::x1
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
    BrenMark2,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Shot {
    x1,
    x2,
    x3,
    x4,
    x5,
    x6,
    x7,
    x8,
    x9,
    x10,
    x11,
    x12,
    x13,
    x14,
    x15,
    x16,
}
impl Shot {
    pub fn count(&self) -> usize {
        match self {
            Shot::x1 => 1,
            Shot::x2 => 2,
            Shot::x3 => 3,
            Shot::x4 => 4,
            Shot::x5 => 5,
            Shot::x6 => 6,
            Shot::x7 => 7,
            Shot::x8 => 8,
            Shot::x9 => 9,
            Shot::x10 => 10,
            Shot::x11 => 11,
            Shot::x12 => 12,
            Shot::x13 => 13,
            Shot::x14 => 14,
            Shot::x15 => 15,
            Shot::x16 => 16,
        }
    }
}

#[derive(Debug)]
pub struct ShotFromIntError(usize);

impl TryFrom<usize> for Shot {
    type Error = ShotFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Shot::x1),
            2 => Ok(Shot::x2),
            3 => Ok(Shot::x3),
            4 => Ok(Shot::x4),
            5 => Ok(Shot::x5),
            6 => Ok(Shot::x6),
            7 => Ok(Shot::x7),
            8 => Ok(Shot::x8),
            9 => Ok(Shot::x9),
            10 => Ok(Shot::x10),
            11 => Ok(Shot::x11),
            12 => Ok(Shot::x12),
            13 => Ok(Shot::x13),
            14 => Ok(Shot::x14),
            15 => Ok(Shot::x15),
            16 => Ok(Shot::x16),
            _ => Err(ShotFromIntError(value)),
        }
    }
}

impl GunFireSoundType {
    pub fn fire_sounds(&self, shots: &Shot) -> Vec<Sound> {
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
            GunFireSoundType::BrenMark2 => match shots {
                Shot::x1 => vec![Sound::BrenMark2x1],
                Shot::x2 => vec![Sound::BrenMark2x2],
                Shot::x3 => vec![Sound::BrenMark2x3],
                Shot::x4 => vec![Sound::BrenMark2x4],
                Shot::x5 => vec![Sound::BrenMark2x5],
                Shot::x6 => vec![Sound::BrenMark2x6],
                Shot::x7 => vec![Sound::BrenMark2x7],
                Shot::x8 => vec![Sound::BrenMark2x8],
                Shot::x9 => vec![Sound::BrenMark2x9],
                Shot::x10 => vec![Sound::BrenMark2x10],
                Shot::x11 => vec![Sound::BrenMark2x11],
                Shot::x12 => vec![Sound::BrenMark2x12],
                Shot::x13 => vec![Sound::BrenMark2x13],
                Shot::x14 => vec![Sound::BrenMark2x14],
                Shot::x15 => vec![Sound::BrenMark2x15],
                Shot::x16 => vec![Sound::BrenMark2x16],
            },
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
    BrenMark2(Option<Magazine>),
}

impl Weapon {
    pub fn name(&self) -> &str {
        match self {
            Weapon::MosinNagantM1924(_, _) => "Mosin Nagant M1924",
            Weapon::MauserG41(_, _) => "Mauser G41",
            Weapon::BrenMark2(_) => "Bren Mark2",
        }
    }

    pub fn gun_fire_sound_type(&self) -> GunFireSoundType {
        match self {
            Weapon::MosinNagantM1924(_, _) => GunFireSoundType::MosinNagant,
            Weapon::MauserG41(_, _) => GunFireSoundType::MauserRiffle,
            Weapon::BrenMark2(_) => GunFireSoundType::BrenMark2,
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
            Weapon::BrenMark2(_) => vec![Sound::ReloadGeneric1],
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
            Weapon::BrenMark2(magazine) => magazine,
        }
    }

    pub fn accepted_magazine(&self, magazine: &Magazine) -> bool {
        match self {
            Weapon::MosinNagantM1924(_, _) => matches!(magazine, Magazine::MosinNagant(_)),
            Weapon::MauserG41(_, _) => matches!(magazine, Magazine::Mauser(_)),
            Weapon::BrenMark2(_) => matches!(magazine, Magazine::BrenCurved30(_)),
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
            Weapon::BrenMark2(_) => Ammunition::x303British,
        }
    }

    pub fn can_fire(&self) -> bool {
        match self {
            Weapon::MosinNagantM1924(ammunition, _) => *ammunition,
            Weapon::MauserG41(ammunition, _) => *ammunition,
            Weapon::BrenMark2(magazine) => magazine
                .as_ref()
                .unwrap_or(&Magazine::BrenCurved30(0))
                .filled(),
        }
    }

    // TODO: not working for all weapon rifle/machine guns
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
            // No bullet reload
            Weapon::BrenMark2(_) => {}
        }

        false
    }

    // TODO: not working for all weapon rifle/machine guns
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
            Weapon::BrenMark2(_) => {}
        }
    }

    pub fn shot(&mut self, shot: &Shot) {
        match self {
            Weapon::MosinNagantM1924(ready_bullet, _) => *ready_bullet = false,
            Weapon::MauserG41(ready_bullet, _) => *ready_bullet = false,
            Weapon::BrenMark2(magazine) => {
                if let Some(magazine) = magazine {
                    magazine.remove(shot.count())
                }
            }
        }

        match self {
            Weapon::MosinNagantM1924(_, magazine)
            | Weapon::MauserG41(_, magazine)
            | Weapon::BrenMark2(magazine) => {
                if let Some(magazine_) = magazine {
                    if !magazine_.filled() {
                        *magazine = None
                    }
                }
            }
        }
    }

    pub fn set_magazine(&mut self, new_magazine: Magazine) {
        match self {
            Weapon::MosinNagantM1924(_, magazine) => *magazine = Some(new_magazine),
            Weapon::MauserG41(_, magazine) => *magazine = Some(new_magazine),
            Weapon::BrenMark2(magazine) => *magazine = Some(new_magazine),
        }
    }

    pub fn ok_count_magazines(&self) -> usize {
        match self {
            Weapon::MosinNagantM1924(_, _) => 5,
            Weapon::MauserG41(_, _) => 5,
            Weapon::BrenMark2(_) => 4,
        }
    }

    pub fn sprite_type(&self) -> WeaponSprite {
        match self {
            Weapon::MosinNagantM1924(_, _) | Weapon::MauserG41(_, _) => WeaponSprite::Riffle,
            // FIXME: machine gun sprite
            Weapon::BrenMark2(_) => WeaponSprite::Riffle,
        }
    }

    pub fn shot_type(&self, opponents_count: usize) -> Shot {
        match self {
            Weapon::MosinNagantM1924(_, magazine) | Weapon::MauserG41(_, magazine) => {
                if magazine.is_some() {
                    Shot::x1
                } else {
                    eprintln!("Tried to determine shot on weapon without magazine");
                    Shot::x1
                }
            }
            Weapon::BrenMark2(magazine) => {
                if let Some(magazine) = magazine {
                    match opponents_count {
                        5.. => magazine.very_long_shot(),
                        3..=4 => magazine.long_shot(),
                        1..=2 => magazine.medium_shot(),
                        _ => magazine.short_shot(),
                    }
                } else {
                    eprintln!("Tried to determine shot on weapon without magazine");
                    Shot::x1
                }
            }
        }
    }

    pub fn frame_offset_on_burst(&self) -> u64 {
        match self {
            Weapon::MosinNagantM1924(_, _) | Weapon::MauserG41(_, _) => 0,
            Weapon::BrenMark2(_) => (TARGET_FPS as f32 / (500. / 60.)) as u64,
        }
    }

    pub fn range_on_burst(&self) -> f32 {
        // FIXME: depend on experience, tiredness, etc
        match self {
            Weapon::MosinNagantM1924(_, _) | Weapon::MauserG41(_, _) => 1.,
            Weapon::BrenMark2(_) => 1.05,
        }
    }

    pub fn reloading_frames(&self) -> u64 {
        match self {
            Weapon::MosinNagantM1924(_, _) => TARGET_FPS,
            Weapon::MauserG41(_, _) => TARGET_FPS,
            Weapon::BrenMark2(_) => TARGET_FPS * 3,
        }
    }

    pub fn aiming_frames(&self) -> u64 {
        match self {
            Weapon::MosinNagantM1924(_, _) => TARGET_FPS / 2,
            Weapon::MauserG41(_, _) => TARGET_FPS / 2,
            Weapon::BrenMark2(_) => TARGET_FPS,
        }
    }

    pub fn firing_frames(&self) -> u64 {
        match self {
            Weapon::MosinNagantM1924(_, _) => TARGET_FPS / 5,
            Weapon::MauserG41(_, _) => TARGET_FPS / 5,
            // FIXME: according to Shot type
            Weapon::BrenMark2(_) => TARGET_FPS,
        }
    }
}
