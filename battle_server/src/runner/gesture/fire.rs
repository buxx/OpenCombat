use battle_core::{
    behavior::gesture::{Gesture, GestureContext},
    entity::soldier::{Soldier, WeaponClass},
    game::weapon::Weapon,
    physics::{utils::distance_between_points, visibility::Visibility},
    types::WorldPoint,
};
use glam::Vec2;
use rand::Rng;

use crate::runner::Runner;

impl Runner {
    pub fn soldier_able_to_fire_on_point<'a>(
        &'a self,
        soldier: &'a Soldier,
        point: &WorldPoint,
    ) -> Option<(WeaponClass, &Weapon, Visibility)> {
        let visibility = self.battle_state.point_is_visible_by_soldier(
            &self.config,
            soldier,
            point,
            // Shoot a hidden point is possible (like fire through a wall)
            self.config.visibility_by_last_frame_shoot_distance,
        );

        if visibility.blocked {
            return None;
        }

        if let Some((weapon_class, weapon)) = self.soldier_weapon_for_point(soldier, point) {
            if weapon.can_fire() || weapon.can_reload() {
                return Some((weapon_class, weapon, visibility));
            }

            if self.soldier_can_reload_with(soldier, weapon).is_some() {
                return Some((weapon_class, weapon, visibility));
            }
        }

        None
    }

    pub fn engage_point_gesture(
        &self,
        soldier: &Soldier,
        engagement: (WeaponClass, &Weapon, Visibility),
    ) -> (GestureContext, Gesture) {
        let frame_i = self.battle_state.frame_i();
        let current = soldier.gesture();
        let (weapon_class, weapon, visibility) = engagement;
        let gesture = match current {
            Gesture::Idle => {
                if weapon.can_fire() {
                    Gesture::Aiming(
                        self.soldier_aiming_end(soldier, weapon),
                        weapon_class.clone(),
                    )
                } else {
                    Gesture::Reloading(
                        self.soldier_reloading_end(soldier, weapon),
                        weapon_class.clone(),
                    )
                }
            }
            Gesture::Reloading(_, _) => {
                //
                current.next(
                    *frame_i,
                    Gesture::Aiming(
                        self.soldier_aiming_end(soldier, weapon),
                        weapon_class.clone(),
                    ),
                )
            }
            Gesture::Aiming(_, _) => {
                //
                let end = self.soldier_firing_end(soldier, weapon);
                current.next(*frame_i, Gesture::Firing(end, weapon_class.clone()))
            }
            Gesture::Firing(_, _) => {
                //
                current.next(*frame_i, Gesture::Idle)
            }
        };

        let final_point = self.soldier_fire_point(soldier, &weapon_class, &visibility.altered_to);
        (
            GestureContext::Firing(final_point, None, visibility),
            gesture,
        )
    }

    // FIXME : use realistic range error (angle from target)
    pub fn soldier_fire_point(
        &self,
        soldier: &Soldier,
        _weapon_class: &WeaponClass,
        target_point: &WorldPoint,
    ) -> WorldPoint {
        let mut rng = rand::thread_rng();
        // TODO : change precision according to weapon, stress, distance, etc
        let factor_by_meter = self.config.inaccurate_fire_factor_by_meter;
        let distance = distance_between_points(&soldier.world_point(), target_point);
        let range = distance.meters() as f32 * factor_by_meter;

        if range == 0. {
            eprintln!(
                "ERROR : soldier_fire_point on original soldier point ({:?})",
                target_point
            );
            return *target_point;
        }

        let x_change = rng.gen_range(-range..range);
        let y_change = rng.gen_range(-range..range);

        target_point.apply(Vec2::new(x_change, y_change))
    }
}
