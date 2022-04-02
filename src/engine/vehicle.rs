use crate::{entity::vehicle::OnBoardPlace, message::*, types::*, utils::angle};

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

        let position = vehicle.get_world_point();
        dbg!((position, move_target_point, angle));

        // FIXME : fake, it must be progressive ...
        vec![Message::SharedState(SharedStateMessage::Vehicle(
            vehicle_index,
            VehicleMessage::SetOrientation(angle),
        ))]
    }
}
