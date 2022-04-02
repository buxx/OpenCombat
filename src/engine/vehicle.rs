use crate::{
    entity::vehicle::OnBoardPlace,
    message::*,
    types::*,
    utils::{angle, short_angle_way},
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
        let position = vehicle.get_world_point();
        let move_target_point = paths
            .next_path_last_point()
            .expect("Execute drive update imply move path is filled");
        let angle = angle(&move_target_point, &position);

        // Vehicle match with next point direction ?
        dbg!(
            angle,
            (vehicle.get_orientation().0 * 100.0).round().abs(),
            (angle.0 * 100.).round().abs()
        );
        if (vehicle.get_orientation().0 * 100.0).round().abs() == (angle.0 * 100.).round().abs() {
            vec![]
        } else {
            let new_orientation = match short_angle_way(vehicle.get_orientation(), &angle) {
                crate::utils::AngleWay::ClockWise => {
                    *vehicle.get_orientation() + vehicle.get_type().rotation_speed()
                }
                crate::utils::AngleWay::CounterClockWise => {
                    *vehicle.get_orientation() + (-vehicle.get_type().rotation_speed())
                }
            };
            vec![Message::SharedState(SharedStateMessage::Vehicle(
                vehicle_index,
                VehicleMessage::SetOrientation(new_orientation),
            ))]
        }
    }
}
