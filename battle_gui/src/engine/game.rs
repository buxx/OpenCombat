use battle_core::{game::Side, types::WorldPoint};
use oc_core::spawn::SpawnZoneName;

use super::Engine;

impl Engine {
    pub fn allowed_drop_point(&self, point: &WorldPoint) -> bool {
        let (allowed_zone_names, opponent_zone_names) = self.zone_controls();

        let in_allowed_zone =
            self.battle_state
                .map()
                .point_in_spawn_zones(point, allowed_zone_names, true);
        let in_opponent_zone =
            self.battle_state
                .map()
                .point_in_spawn_zones(point, opponent_zone_names, false);

        in_allowed_zone && !in_opponent_zone
    }

    pub fn zone_controls(&self) -> (&Vec<SpawnZoneName>, &Vec<SpawnZoneName>) {
        if self.gui_state.side() == &Side::A {
            (&self.a_control, &self.b_control)
        } else {
            (&self.b_control, &self.a_control)
        }
    }
}
