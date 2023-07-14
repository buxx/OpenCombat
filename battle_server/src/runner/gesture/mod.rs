use battle_core::{
    behavior::{
        gesture::{Gesture, GestureContext},
        Behavior,
    },
    entity::soldier::{Soldier, WeaponClass},
    game::weapon::Weapon,
    physics::event::bullet::BulletFire,
    state::{
        battle::message::{BattleStateMessage, SoldierMessage},
        client::ClientStateMessage,
    },
    types::{Precision, SoldierIndex, WorldPoint},
};

use super::{message::RunnerMessage, Runner};

mod engage;
mod fire;
mod soldier;
mod suppress;
mod weapon;

impl Runner {
    pub fn soldier_gesture(&self, soldier: &Soldier) -> Vec<RunnerMessage> {
        puffin::profile_scope!("soldier_gesture");
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
                vec![RunnerMessage::BattleState(BattleStateMessage::Soldier(
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
    ) -> Vec<RunnerMessage> {
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
    ) -> Vec<RunnerMessage> {
        [
            vec![RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier.uuid(),
                SoldierMessage::ReloadWeapon(class.clone()),
            ))],
            weapon
                .reload_sounds()
                .iter()
                .map(|sound| {
                    RunnerMessage::ClientsState(ClientStateMessage::PlayBattleSound(*sound))
                })
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
    ) -> Vec<RunnerMessage> {
        [vec![
            RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier.uuid(),
                SoldierMessage::WeaponShot(class.clone()),
            )),
            RunnerMessage::BattleState(BattleStateMessage::PushBulletFire(BulletFire::new(
                soldier.world_point(),
                *point,
                target.clone(),
                weapon.ammunition(),
                weapon.gun_fire_sound_type(),
            ))),
            RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier.uuid(),
                SoldierMessage::SetLastShootFrameI(self.frame_i),
            )),
        ]]
        .concat()
    }
}
