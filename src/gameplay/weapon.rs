use crate::audio::Sound;
use crate::{Factor, Meters};

#[derive(Clone, Copy)]
pub enum SceneItemWeapon {
    MainWeapon,
}

#[derive(Clone, Copy)]
pub enum WeaponType {
    GarandM1,
    MauserG41,
    MosinNagant1938,
}

pub struct Weapon {
    pub type_: WeaponType,
    pub need_reload: bool,
}

impl Weapon {
    pub fn new(type_: WeaponType) -> Self {
        Self {
            type_,
            need_reload: false,
        }
    }

    pub fn characteristics(&self) -> WeaponCharacteristic {
        WeaponCharacteristic::new(&self.type_)
    }
}

pub struct WeaponCharacteristic {
    pub miss_by_distance_factors: Vec<(Meters, Factor)>,
    pub sound: Sound,
    pub minimal_auto_engage_distance: Meters,
    pub maximal_auto_engage_distance: Meters,
}

impl WeaponCharacteristic {
    pub fn new(type_: &WeaponType) -> Self {
        // TODO: perf ?
        let default_miss_by_distance_factors: Vec<(Meters, Factor)> = vec![
            (0.0, 0.5),
            (100.0, 1.0),
            (150.0, 1.5),
            (250.0, 2.0),
            (300.0, 4.0),
            (500.0, 10.0),
        ];

        match type_ {
            WeaponType::GarandM1 => Self {
                miss_by_distance_factors: default_miss_by_distance_factors,
                sound: Sound::GarandM1SingleShot,
                minimal_auto_engage_distance: 0.0,
                maximal_auto_engage_distance: 350.0,
            },
            WeaponType::MauserG41 => Self {
                miss_by_distance_factors: default_miss_by_distance_factors,
                sound: Sound::GarandM1SingleShot,
                minimal_auto_engage_distance: 0.0,
                maximal_auto_engage_distance: 350.0,
            },
            WeaponType::MosinNagant1938 => Self {
                miss_by_distance_factors: default_miss_by_distance_factors,
                sound: Sound::GarandM1SingleShot,
                minimal_auto_engage_distance: 0.0,
                maximal_auto_engage_distance: 350.0,
            },
        }
    }
}
