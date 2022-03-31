use ggez::GameResult;

use super::shared::SharedState;

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
}
