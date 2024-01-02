use battle_core::{
    behavior::{
        gesture::{Gesture, GestureContext},
        Behavior,
    },
    entity::soldier::{Soldier, WeaponClass},
    game::weapon::Weapon,
    physics::event::{bullet::BulletFire, cannon_blast::CannonBlast},
    state::{
        battle::message::{BattleStateMessage, SoldierMessage},
        client::ClientStateMessage,
    },
    types::{Precision, SoldierIndex, WorldPoint},
};

use super::{message::RunnerMessage, Runner};

mod engage;
mod fire;
mod idle;
mod soldier;
mod suppress;
mod weapon;

pub struct FallbackBehavior(pub Behavior);

pub enum GestureResult {
    Handled(GestureContext, Gesture),
    Cant(Option<FallbackBehavior>),
}

impl Runner {
    pub fn soldier_gesture(&self, soldier: &Soldier) -> Vec<RunnerMessage> {
        puffin::profile_scope!("soldier_gesture");
        let mut messages = vec![];

        let new_gesture = match soldier.behavior() {
            Behavior::Idle(_) => {
                //
                self.idle_gesture(soldier)
            }
            Behavior::SuppressFire(point) => {
                //
                self.suppress_fire_gesture(soldier, point)
            }
            Behavior::EngageSoldier(soldier_index) => {
                //
                self.engage_soldier_gesture(soldier, soldier_index)
            }
            _ => GestureResult::Handled(GestureContext::Idle, Gesture::Idle),
        };

        match new_gesture {
            GestureResult::Handled(context, gesture) => {
                if &gesture != soldier.gesture() {
                    return [
                        self.new_gesture_messages(soldier, &context, &gesture),
                        vec![RunnerMessage::BattleState(BattleStateMessage::Soldier(
                            soldier.uuid(),
                            SoldierMessage::SetGesture(gesture),
                        ))],
                    ]
                    .concat();
                }
            }
            GestureResult::Cant(fallback) => {
                if let Some(fallback) = fallback {
                    messages.push(RunnerMessage::BattleState(BattleStateMessage::Soldier(
                        soldier.uuid(),
                        SoldierMessage::SetBehavior(fallback.0),
                    )));
                }
            }
        }

        messages
    }

    fn new_gesture_messages(
        &self,
        soldier: &Soldier,
        context: &GestureContext,
        gesture: &Gesture,
    ) -> Vec<RunnerMessage> {
        match (context, gesture) {
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
            RunnerMessage::BattleState(BattleStateMessage::PushCannonBlast(CannonBlast::new(
                soldier.world_point(),
                soldier.get_looking_direction(),
                weapon.sprite_type(),
                soldier.animation_type().0,
            ))),
            RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier.uuid(),
                SoldierMessage::SetLastShootFrameI(*self.battle_state.frame_i()),
            )),
        ]]
        .concat()
    }
}
