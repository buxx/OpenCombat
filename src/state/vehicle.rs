use ggez::GameResult;

use crate::{
    graphics::vehicle::VehicleGraphicInfos, message::VehicleMessage, types::*,
    utils::apply_angle_on_point,
};

use super::{shared::SharedState, SideEffect};

impl SharedState {
    pub fn check_board_integrity(&self) -> GameResult {
        // Check if on boards are coherent with vehicle types
        for (vehicle_index, board) in self.vehicle_board() {
            let vehicle = self.vehicle(*vehicle_index);
            let board_composition = vehicle.get_type().board_composition();

            for (place, _) in board {
                if !board_composition.contains(&place) {
                    panic!(
                        "Vehicle {:?} has a board with place {:?} which is not in its composition",
                        vehicle_index, place
                    );
                }
            }
        }

        Ok(())
    }

    pub fn react_vehicle_message(
        &mut self,
        vehicle_index: VehicleIndex,
        vehicle_message: VehicleMessage,
    ) -> Vec<SideEffect> {
        if !self.initialized() {
            return vec![];
        }

        let vehicle = &mut self.vehicle_mut(vehicle_index);
        match vehicle_message {
            VehicleMessage::SetWorldPosition(new_world_point) => {
                vehicle.set_world_point(new_world_point);
                self.propagate_vehicle_position(vehicle_index);
            }
            VehicleMessage::SetChassisOrientation(angle) => {
                vehicle.set_chassis_orientation(angle);
                self.propagate_vehicle_position(vehicle_index);
            }
        }

        vec![]
    }

    pub fn propagate_vehicle_position(&mut self, vehicle_index: VehicleIndex) {
        let vehicle = &mut self.vehicle_mut(vehicle_index);
        let vehicle_point = vehicle.get_world_point();
        let vehicle_orientation = vehicle.get_chassis_orientation().clone();
        let sprite_infos = VehicleGraphicInfos::sprites_infos(vehicle.get_type());
        let places = sprite_infos.places();
        let mut new_positions: Vec<(SoldierIndex, WorldPoint)> = vec![];
        for (place, soldier_index) in self.vehicle_board().get(&vehicle_index).unwrap_or(&vec![]) {
            let place_offset = places
                .get(place)
                .expect("Vehicle place position coherence must be check at startup");
            let place_point =
                WorldPoint::from_vec2(vehicle_point.to_vec2() + place_offset.to_vec2());
            // Modify according to vehicle orientation
            let place_point =
                apply_angle_on_point(&place_point, &vehicle_point, &vehicle_orientation);

            new_positions.push((*soldier_index, place_point));
        }

        for (soldier_index, world_point) in new_positions {
            let soldier = self.soldier_mut(soldier_index);
            soldier.set_world_point(world_point)
        }
    }

    pub fn initialize_vehicle_positions(&mut self) {
        for i in 0..self.vehicles().len() {
            self.propagate_vehicle_position(VehicleIndex(i))
        }
    }
}
