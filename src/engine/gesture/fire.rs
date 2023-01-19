use crate::{
    engine::Engine,
    entity::soldier::{Soldier, WeaponClass},
    game::weapon::Weapon,
    types::WorldPoint,
};
use glam::Vec2;
use rand::Rng;

impl Engine {
    pub fn soldier_able_to_fire_on_point<'a>(
        &'a self,
        soldier: &'a Soldier,
        point: &WorldPoint,
    ) -> Option<(WeaponClass, &Weapon)> {
        if !self.point_is_visible_by_soldier(soldier, point) {
            return None;
        }

        if let Some((weapon_class, weapon)) = self.soldier_weapon_for_point(soldier, point) {
            if weapon.can_fire() || weapon.can_reload_and_shoot() {
                return Some((weapon_class, weapon));
            }

            if self.soldier_can_reload_with(soldier, weapon).is_some() {
                return Some((weapon_class, weapon));
            }
        }

        None
    }

    pub fn soldier_fire_point(
        &self,
        _soldier: &Soldier,
        _weapon_class: &WeaponClass,
        target_point: &WorldPoint,
    ) -> WorldPoint {
        let mut rng = rand::thread_rng();
        // TODO : change precision according to weapon, stress, distance, etc
        let x_change = rng.gen_range(-10.0..10.0);
        let y_change = rng.gen_range(-10.0..10.0);
        WorldPoint::from(target_point.apply(Vec2::new(x_change, y_change)))
    }
}
