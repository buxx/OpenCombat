use battle_core::{
    entity::soldier::{Soldier, WeaponClass},
    game::weapon::{Magazine, Weapon},
    types::WorldPoint,
};

use crate::runner::Runner;

impl Runner {
    pub fn soldier_weapon_for_point<'a>(
        &'a self,
        soldier: &'a Soldier,
        _point: &WorldPoint,
    ) -> Option<(WeaponClass, &Weapon)> {
        // TODO : according to distance, choose weapon
        if let Some(weapon) = soldier.main_weapon() {
            Some((WeaponClass::Main, weapon))
        } else {
            None
        }
    }

    pub fn soldier_can_reload_with<'a>(
        &'a self,
        soldier: &'a Soldier,
        weapon: &Weapon,
    ) -> Option<&Magazine> {
        for magazine in soldier.magazines() {
            if weapon.accepted_magazine(magazine) {
                return Some(magazine);
            }
        }

        None
    }
}
