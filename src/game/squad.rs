use std::collections::HashMap;

use crate::{state::shared::SharedState, types::*, utils::apply_angle_on_point};

pub enum Formation {
    Line,
}

pub fn squad_positions(
    squad: &SquadComposition,
    formation: Formation,
    state: &SharedState,
) -> HashMap<SoldierIndex, WorldPoint> {
    let mut positions = HashMap::new();
    let squad_leader = state.soldier(squad.leader());
    let ref_point = squad_leader.get_world_point();
    let ref_angle = squad_leader.get_looking_direction();

    match formation {
        Formation::Line => {
            let mut x_offset: f32 = 0.0;
            let mut y_offset: f32 = 0.0;
            let mut counter: u8 = 0;

            for (i, soldier_index) in squad.members().iter().enumerate() {
                // Don't return position for leader
                if *soldier_index == squad.leader() {
                    continue;
                }

                if counter % 2 == 0 {
                    x_offset += 10.0;
                    y_offset += 0.0;
                }
                counter += 1;

                let (x_offset_, y_offset_) = if i % 2 == 0 {
                    (x_offset, y_offset)
                } else {
                    (-x_offset, -y_offset)
                };

                let member_scene_point =
                    WorldPoint::new(ref_point.x + x_offset_, ref_point.y + y_offset_);
                let member_scene_point =
                    apply_angle_on_point(&member_scene_point, &ref_point, &ref_angle);
                positions.insert(*soldier_index, member_scene_point);
            }
        }
    }

    positions
}
