use crate::{
    behavior::Behavior,
    config::{
        VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_COEFFICIENT,
        VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_DIFF,
        VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT,
    },
    entity::vehicle::OnBoardPlace,
    message::*,
    types::*,
    utils::{angle, short_angle, short_angle_way},
};

use super::Engine;

impl Engine {
    pub fn soldier_vehicle_place(&self, soldier_index: SoldierIndex) -> Option<&OnBoardPlace> {
        if let Some((_, place)) = self.shared_state.soldier_board(soldier_index) {
            return Some(place);
        }
        None
    }

    pub fn drive_update(&self, soldier_index: SoldierIndex, paths: &WorldPaths) -> Vec<Message> {
        let vehicle_index = self
            .shared_state
            .soldier_board(soldier_index)
            .expect("this code must be called only when soldier is on board")
            .0;
        let vehicle = self.shared_state.vehicle(vehicle_index);
        let vehicle_position = vehicle.get_world_point();
        let move_target_point = paths
            .next_path_last_point()
            .expect("Execute drive update imply move path is filled");
        let angle = angle(&move_target_point, &vehicle_position);
        let move_vector = (move_target_point.to_vec2() - vehicle_position.to_vec2()).normalize()
            * vehicle.get_type().drive_speed();

        let rounded_vehicle_orientation = (vehicle.get_orientation().0
            * VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT)
            .round()
            .abs();
        let target_vehicle_orientation = (angle.0
            * VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT)
            .round()
            .abs();

        let mut messages = vec![];

        // Need to rotate ?
        if rounded_vehicle_orientation != target_vehicle_orientation {
            let new_orientation = match short_angle_way(vehicle.get_orientation(), &angle) {
                crate::utils::AngleWay::ClockWise => {
                    *vehicle.get_orientation() + vehicle.get_type().rotation_speed()
                }
                crate::utils::AngleWay::CounterClockWise => {
                    *vehicle.get_orientation() + (-vehicle.get_type().rotation_speed())
                }
            };
            messages.push(Message::SharedState(SharedStateMessage::Vehicle(
                vehicle_index,
                VehicleMessage::SetOrientation(new_orientation),
            )));
        }

        // Can advance ?
        if (short_angle(vehicle.get_orientation(), &angle).0
            * VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_COEFFICIENT)
            .abs()
            < VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_DIFF
        {
            let new_point = vehicle_position.apply(move_vector);
            messages.push(Message::SharedState(SharedStateMessage::Vehicle(
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
                messages.push(Message::SharedState(SharedStateMessage::Soldier(
                    soldier_index,
                    SoldierMessage::SetBehavior(Behavior::Idle),
                )));
            } else {
                messages.push(Message::SharedState(SharedStateMessage::Soldier(
                    soldier_index,
                    SoldierMessage::ReachBehaviorStep,
                )));
            }
        }

        messages
    }
}
