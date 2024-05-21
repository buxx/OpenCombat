use battle_core::{
    behavior::{
        gesture::{Gesture, GestureContext},
        Behavior,
    },
    entity::soldier::{Soldier, WeaponClass},
    game::{
        weapon::{Shot, Weapon},
        Side,
    },
    physics::{
        event::{bullet::BulletFire, cannon_blast::CannonBlast},
        utils::distance_between_points,
        visibility::Visibility,
    },
    state::{
        battle::message::{BattleStateMessage, SoldierMessage},
        client::ClientStateMessage,
    },
    types::{Distance, Precision, SoldierIndex, WorldPoint},
};
use glam::Vec2;
use rand::Rng;

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
            (GestureContext::Firing(point, target, visibility), Gesture::Firing(_, class)) => {
                if let Some(weapon) = soldier.weapon(class) {
                    return self.firing_gesture_messages(
                        soldier, class, weapon, point, target, visibility,
                    );
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
        visibility: &Visibility,
    ) -> Vec<RunnerMessage> {
        let mut rng = rand::thread_rng();
        // TODO: value in config
        let opponents_around = self.count_opponents_around(
            &soldier.side().opposite(),
            &visibility.to,
            Distance::from_meters(5),
        );
        let shot = weapon.shot_type(opponents_around);

        // FIXME BS NOW: generate multiple BulletFire & CannonBlast
        // FIXME BS NOW: warn about sound !

        let bullet_fires = (0..shot.count())
            .map(|i| {
                let point = if i > 0 {
                    let weapon_factor_multiplier = weapon.range_on_burst();
                    let factor_by_meter = self.config.inaccurate_fire_factor_by_meter;
                    let distance = visibility.distance;
                    let range =
                        distance.meters() as f32 * factor_by_meter * weapon_factor_multiplier;
                    if range > 0. {
                        let x_change = rng.gen_range(-range..range);
                        let y_change = rng.gen_range(-range..range);
                        point.apply(Vec2::new(x_change, y_change))
                    } else {
                        *point
                    }
                } else {
                    *point
                };
                let sound = if i == 0 {
                    Some(weapon.gun_fire_sound_type())
                } else {
                    None
                };
                RunnerMessage::BattleState(BattleStateMessage::PushBulletFire(BulletFire::new(
                    weapon.frame_offset_on_burst() * i as u64,
                    soldier.world_point(),
                    point,
                    target.clone(),
                    weapon.ammunition(),
                    sound,
                    shot,
                )))
            })
            .collect();

        [
            vec![
                RunnerMessage::BattleState(BattleStateMessage::Soldier(
                    soldier.uuid(),
                    SoldierMessage::WeaponShot(class.clone(), shot),
                )),
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
            ],
            bullet_fires,
        ]
        .concat()
    }

    fn count_opponents_around(&self, side: &Side, point: &WorldPoint, distance: Distance) -> usize {
        self.battle_state
            .soldiers()
            .iter()
            .filter(|s| s.side() == side)
            .filter(|s| s.can_be_designed_as_target())
            .filter(|s| {
                distance_between_points(&s.world_point(), point).millimeters()
                    <= distance.millimeters()
            })
            .collect::<Vec<&Soldier>>()
            .len()
    }
}
