use crate::{
    behavior::gesture::{Gesture, GestureContext},
    engine::Engine,
    entity::soldier::{Soldier, WeaponClass},
    game::weapon::Weapon,
    types::WorldPoint,
};

impl Engine {
    pub fn engage_point_gesture(
        &self,
        soldier: &Soldier,
        point: &WorldPoint,
        weapon: (WeaponClass, &Weapon),
    ) -> (GestureContext, Gesture) {
        let frame_i = self.local_state.get_frame_i();
        let current = soldier.gesture();

        let gesture = match current {
            Gesture::Idle => {
                //
                Gesture::Reloading(
                    self.soldier_reloading_end(soldier, weapon.1),
                    weapon.0.clone(),
                )
            }
            Gesture::Reloading(_, _) => {
                //
                current.next(
                    frame_i,
                    Gesture::Aiming(self.soldier_aiming_end(soldier, weapon.1), weapon.0.clone()),
                )
            }
            Gesture::Aiming(_, _) => {
                //
                let end = self.soldier_firing_end(soldier, weapon.1);
                current.next(frame_i, Gesture::Firing(end, weapon.0.clone()))
            }
            Gesture::Firing(_, _) => {
                //
                current.next(frame_i, Gesture::Idle)
            }
        };

        let final_point = self.soldier_fire_point(soldier, &weapon.0, point);
        (GestureContext::Firing(final_point, None), gesture)
    }
}
