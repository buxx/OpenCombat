pub enum WeaponType {
    GarandM1,
    MauserG41,
    MosinNagantM18911930,
}

pub struct Weapon {
    pub type_: WeaponType,
}

impl Weapon {
    pub fn new(type_: WeaponType) -> Self {
        Self { type_ }
    }
}
