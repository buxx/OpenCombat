use battle_core::{
    behavior::Behavior,
    config::{
        VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_COEFFICIENT,
        VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_DIFF,
        VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT,
    },
    state::battle::message::{BattleStateMessage, SoldierMessage, VehicleMessage},
    types::*,
    utils::{angle, short_angle, short_angle_way, AngleWay},
};

use super::{message::RunnerMessage, Runner};

impl Runner {
    pub fn drive_update(
        &self,
        soldier_index: SoldierIndex,
        paths: &WorldPaths,
    ) -> Vec<RunnerMessage> {
        let vehicle_index = self
            .battle_state
            .soldier_board(soldier_index)
            .expect("this code must be called only when soldier is on board")
            .0;
        let vehicle = self.battle_state.vehicle(vehicle_index);
        let vehicle_position = vehicle.get_world_point();
        let move_target_point = paths
            .next_point()
            .expect("Execute drive update imply move path is filled");
        let angle = angle(&move_target_point, &vehicle_position);
        let move_vector = (move_target_point.to_vec2() - vehicle_position.to_vec2()).normalize()
            * vehicle.get_type().drive_speed();

        let rounded_chassis_orientation = (vehicle.get_chassis_orientation().0
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
            let new_orientation = match short_angle_way(vehicle.get_chassis_orientation(), &angle) {
                AngleWay::ClockWise => {
                    *vehicle.get_chassis_orientation() + vehicle.get_type().chassis_rotation_speed()
                }
                AngleWay::CounterClockWise => {
                    *vehicle.get_chassis_orientation()
                        + (-vehicle.get_type().chassis_rotation_speed())
                }
            };
            messages.push(RunnerMessage::BattleState(BattleStateMessage::Vehicle(
                vehicle_index,
                VehicleMessage::SetChassisOrientation(new_orientation),
            )));
        }

        // Can advance ?
        if (short_angle(vehicle.get_chassis_orientation(), &angle).0
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
                    SoldierMessage::SetBehavior(Behavior::Idle),
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
}
