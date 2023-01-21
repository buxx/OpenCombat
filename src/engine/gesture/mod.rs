use crate::{
    behavior::{
        gesture::{Gesture, GestureContext},
        Behavior,
    },
    entity::soldier::{Soldier, WeaponClass},
    game::weapon::Weapon,
    message::{Message, PhysicsMessage, SharedStateMessage, SoldierMessage},
    physics::event::bullet::BulletFire,
    types::{Precision, SoldierIndex, WorldPoint},
};

use super::Engine;

mod engage;
mod fire;
mod soldier;
mod suppress;
mod weapon;

impl Engine {
    pub fn soldier_gesture(&self, soldier: &Soldier) -> Vec<Message> {
        let new_gesture = match soldier.behavior() {
            Behavior::SuppressFire(point) => {
                //
                self.suppress_fire_gesture(soldier, point)
            }
            Behavior::EngageSoldier(soldier_index) => {
                //
                self.engage_soldier_gesture(soldier, soldier_index)
            }
            _ => (GestureContext::Idle, Gesture::Idle),
        };

        if &new_gesture.1 != soldier.gesture() {
            return [
                self.new_gesture_messages(soldier, &new_gesture),
                vec![Message::SharedState(SharedStateMessage::Soldier(
                    soldier.uuid(),
                    SoldierMessage::SetGesture(new_gesture.1),
                ))],
            ]
            .concat();
        }

        vec![]
    }

    fn new_gesture_messages(
        &self,
        soldier: &Soldier,
        gesture: &(GestureContext, Gesture),
    ) -> Vec<Message> {
        match gesture {
            (_, Gesture::Idle) => {}
            (_, Gesture::Reloading(_, class)) => {
                if let Some(weapon) = soldier.weapon(class) {
                    return self.reloading_gesture_messages(soldier, class, weapon);
                }
            }
            (_, Gesture::Aiming(_, _)) => {}
            (GestureContext::Firing(point, target), Gesture::Firing(_, class)) => {
                if let Some(weapon) = soldier.weapon(class) {
                    return self.firing_gesture_messages(soldier, class, weapon, point, target);
                }
            }
            _ => {}
        }

        vec![]
    }

    pub fn reloading_gesture_messages(
        &self,
        soldier: &Soldier,
        class: &WeaponClass,
        weapon: &Weapon,
    ) -> Vec<Message> {
        [
            vec![Message::SharedState(SharedStateMessage::Soldier(
                soldier.uuid(),
                SoldierMessage::ReloadWeapon(class.clone()),
            ))],
            weapon
                .reload_sounds()
                .iter()
                .map(|sound| Message::SharedState(SharedStateMessage::PushSoundToPlay(*sound)))
                .collect(),
        ]
        .concat()
    }

    pub fn firing_gesture_messages(
        &self,
        soldier: &Soldier,
        class: &WeaponClass,
        weapon: &Weapon,
        point: &WorldPoint,
        target: &Option<(SoldierIndex, Precision)>,
    ) -> Vec<Message> {
        [
            vec![
                Message::SharedState(SharedStateMessage::Soldier(
                    soldier.uuid(),
                    SoldierMessage::WeaponShot(class.clone()),
                )),
                Message::Physics(PhysicsMessage::PushBulletFire(BulletFire::new(
                    soldier.get_world_point(),
                    *point,
                    target.clone(),
                    weapon.ammunition(),
                ))),
            ],
            // FIXME BS NOW : faire varier de quelque ms le début du son OR le temps de recharge
            weapon
                .fire_sounds()
                .iter()
                .map(|sound| Message::SharedState(SharedStateMessage::PushSoundToPlay(*sound)))
                .collect(),
        ]
        .concat()
    }
}
