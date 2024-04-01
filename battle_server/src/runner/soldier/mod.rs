pub mod fight;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use battle_core::{
    behavior::{Behavior, Body},
    config::{
        ServerConfig, VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_COEFFICIENT,
        VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_DIFF,
        VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT,
    },
    order::Order,
    state::battle::{
        message::{BattleStateMessage, SoldierMessage, VehicleMessage},
        BattleState,
    },
    types::{SoldierIndex, *},
    utils::{angle, short_angle, short_angle_way, AngleWay},
};

use super::{message::RunnerMessage, utils::behavior_angle};

pub mod behavior;
pub mod engage;
pub mod gesture;

pub struct SoldierRunner {
    config: ServerConfig,
    battle_state: Arc<RwLock<BattleState>>,
}

impl SoldierRunner {
    pub fn new(config: ServerConfig, battle_state: Arc<RwLock<BattleState>>) -> Self {
        Self {
            config,
            battle_state,
        }
    }

    pub fn battle_state(&self) -> RwLockReadGuard<'_, BattleState> {
        // FIXME unwrap
        self.battle_state.read().unwrap()
    }

    // TODO : Soldiers in vehicles must be managed differently than ground soldiers
    pub fn tick_animate_soldiers(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_animate_soldiers");
        let mut messages = vec![];

        messages.extend(
            (0..self.battle_state().soldiers().len())
                // TODO : For now, parallel iter cost more than serial
                // .into_par_iter()
                .flat_map(|i| self.animate_soldier(SoldierIndex(i)))
                .collect::<Vec<RunnerMessage>>(),
        );

        messages
    }

    // TODO : Soldiers in vehicles must be managed differently than ground soldiers
    pub fn tick_update_soldiers(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_update_soldiers");
        let mut messages = vec![];

        let soldier_messages: Vec<RunnerMessage> = (0..self.battle_state().soldiers().len())
            // TODO : For now, parallel iter cost more than serial
            // .into_par_iter()
            .flat_map(|i| self.update_soldier(SoldierIndex(i)))
            .collect();
        messages.extend(soldier_messages);

        messages
    }

    pub fn soldier_is_squad_leader(&self, soldier_index: SoldierIndex) -> bool {
        let soldier = self.battle_state().soldier(soldier_index);
        let squad_uuid = soldier.squad_uuid();
        let squad_composition = self.battle_state().squad(squad_uuid);
        let squad_leader = squad_composition.leader();
        squad_leader == soldier_index
    }

    pub fn animate_soldier(&self, soldier_index: SoldierIndex) -> Vec<RunnerMessage> {
        puffin::profile_scope!("animate_soldier", format!("{}", soldier_index));
        let soldier = self.battle_state().soldier(soldier_index);
        if !soldier.can_be_animated() {
            return vec![];
        }

        let mut messages = vec![];

        messages.extend(self.soldier_behavior(soldier));
        messages.extend(self.soldier_gesture(soldier));

        messages
    }

    pub fn update_soldier(&self, i: SoldierIndex) -> Vec<RunnerMessage> {
        puffin::profile_scope!("update_soldier", format!("{}", i));
        let mut messages = vec![];

        messages.extend(self.orientation_update(i));
        messages.extend(self.behavior_update(i));

        messages
    }

    fn orientation_update(&self, i: SoldierIndex) -> Vec<RunnerMessage> {
        let soldier = self.battle_state().soldier(i);
        let mut messages = vec![];

        if let Some(angle_) = behavior_angle(soldier.behavior(), &soldier.world_point()) {
            let soldier_message = SoldierMessage::SetOrientation(angle_);
            messages.push(RunnerMessage::BattleState(BattleStateMessage::Soldier(
                i,
                soldier_message,
            )));
        }

        messages
    }

    fn behavior_update(&self, soldier_index: SoldierIndex) -> Vec<RunnerMessage> {
        let soldier = self.battle_state().soldier(soldier_index);
        let mut messages = vec![];

        messages.extend(match soldier.behavior() {
            Behavior::Idle(_) => {
                vec![]
            }
            Behavior::MoveTo(paths) | Behavior::MoveFastTo(paths) | Behavior::SneakTo(paths) => {
                self.movement_updates(soldier_index, paths)
            }
            Behavior::Defend(_) => {
                vec![]
            }
            Behavior::Hide(_) => {
                vec![]
            }
            Behavior::DriveTo(paths) => self.drive_update(soldier_index, paths),
            Behavior::RotateTo(angle) => self.rotate_update(soldier_index, angle),
            Behavior::SuppressFire(_) => {
                vec![]
            }
            Behavior::EngageSoldier(target) => self.engage_update(&soldier_index, target),
            Behavior::Dead => vec![],
            Behavior::Unconscious => vec![],
        });

        messages
    }

    pub fn movement_updates(
        &self,
        soldier_index: SoldierIndex,
        path: &WorldPaths,
    ) -> Vec<RunnerMessage> {
        let mut messages = vec![];
        let soldier = self.battle_state().soldier(soldier_index);
        let point = path.next_point().expect("Must have point in path");

        // There is a next point in path, go to it
        let velocity = self
            .config
            .behavior_velocity(soldier.behavior())
            .expect("Entity behavior must have velocity when move code called");
        let vector = (point.to_vec2() - soldier.world_point().to_vec2()).normalize() * velocity;

        // Point reached
        if vector.is_nan()
            || (soldier.world_point().to_vec2() - point.to_vec2()).length() <= vector.length()
        {
            // If it is the last point, move is finished
            if path.is_last_point().expect("Must contain points") {
                let (behavior, order) = if let Some(then_order) = soldier.order().then() {
                    (
                        Behavior::from_order(&then_order, soldier, &self.battle_state()),
                        then_order,
                    )
                } else {
                    (
                        Behavior::Idle(Body::from_soldier(soldier, &self.battle_state())),
                        Order::Idle,
                    )
                };

                messages.extend(vec![
                    RunnerMessage::BattleState(BattleStateMessage::Soldier(
                        soldier_index,
                        SoldierMessage::SetBehavior(behavior),
                    )),
                    RunnerMessage::BattleState(BattleStateMessage::Soldier(
                        soldier_index,
                        SoldierMessage::SetOrder(order),
                    )),
                ]);
            } else {
                messages.push(RunnerMessage::BattleState(BattleStateMessage::Soldier(
                    soldier_index,
                    SoldierMessage::ReachBehaviorStep,
                )));
            }

            // Movement required
        } else {
            let new_point = soldier.world_point().apply(vector);
            messages.push(RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetWorldPosition(new_point),
            )));
        }

        messages
    }

    pub fn drive_update(
        &self,
        soldier_index: SoldierIndex,
        paths: &WorldPaths,
    ) -> Vec<RunnerMessage> {
        let vehicle_index = self
            .battle_state()
            .soldier_board(soldier_index)
            .expect("this code must be called only when soldier is on board")
            .0;
        let vehicle = self.battle_state().vehicle(vehicle_index);
        let vehicle_position = vehicle.world_point();
        let move_target_point = paths
            .next_point()
            .expect("Execute drive update imply move path is filled");
        let angle = angle(&move_target_point, &vehicle_position);
        let move_vector = (move_target_point.to_vec2() - vehicle_position.to_vec2()).normalize()
            * vehicle.type_().drive_speed();

        let rounded_chassis_orientation = (vehicle.chassis_orientation().0
            * VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT)
            .round()
            .abs();
        let target_vehicle_orientation = (angle.0
            * VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT)
            .round()
            .abs();

        let mut messages = vec![];

        // Need to rotate chassis ?
        if rounded_chassis_orientation != target_vehicle_orientation {
            let new_orientation = match short_angle_way(vehicle.chassis_orientation(), &angle) {
                AngleWay::ClockWise => {
                    *vehicle.chassis_orientation() + vehicle.type_().chassis_rotation_speed()
                }
                AngleWay::CounterClockWise => {
                    *vehicle.chassis_orientation() + (-vehicle.type_().chassis_rotation_speed())
                }
            };
            messages.push(RunnerMessage::BattleState(BattleStateMessage::Vehicle(
                vehicle_index,
                VehicleMessage::SetChassisOrientation(new_orientation),
            )));
        }

        // Can advance ?
        if (short_angle(vehicle.chassis_orientation(), &angle).0
            * VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_COEFFICIENT)
            .abs()
            < VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_DIFF
        {
            let new_point = vehicle_position.apply(move_vector);
            messages.push(RunnerMessage::BattleState(BattleStateMessage::Vehicle(
                vehicle_index,
                VehicleMessage::SetWorldPosition(new_point),
            )));
        }

        // Next point reached ?
        if (vehicle_position.to_vec2() - move_target_point.to_vec2()).length()
            <= move_vector.length()
        {
            // If it is the last point, move is finished
            if paths.is_last_point().expect("Must contain points") {
                messages.push(RunnerMessage::BattleState(BattleStateMessage::Soldier(
                    soldier_index,
                    SoldierMessage::SetBehavior(Behavior::Idle(Body::Crouched)),
                )));
            } else {
                messages.push(RunnerMessage::BattleState(BattleStateMessage::Soldier(
                    soldier_index,
                    SoldierMessage::ReachBehaviorStep,
                )));
            }
        }

        messages
    }

    pub fn rotate_update(&self, soldier_index: SoldierIndex, angle: &Angle) -> Vec<RunnerMessage> {
        let vehicle_index = self
            .battle_state()
            .soldier_board(soldier_index)
            .expect("this code must be called only when soldier is on board")
            .0;
        let vehicle = self.battle_state().vehicle(vehicle_index);

        let mut messages = vec![];

        // Need to rotate chassis ?
        if !vehicle.chassis_orientation_match(angle) {
            let new_orientation = match short_angle_way(vehicle.chassis_orientation(), angle) {
                AngleWay::ClockWise => {
                    *vehicle.chassis_orientation() + vehicle.type_().chassis_rotation_speed()
                }
                AngleWay::CounterClockWise => {
                    *vehicle.chassis_orientation() + (-vehicle.type_().chassis_rotation_speed())
                }
            };
            messages.push(RunnerMessage::BattleState(BattleStateMessage::Vehicle(
                vehicle_index,
                VehicleMessage::SetChassisOrientation(new_orientation),
            )));
        } else {
            messages.push(RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetBehavior(Behavior::Idle(Body::Crouched)),
            )))
        }

        messages
    }
}
