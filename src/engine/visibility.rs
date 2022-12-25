use std::collections::HashMap;

use crate::{
    entity::soldier::Soldier,
    game::Side,
    message::{LocalStateMessage, Message},
    physics::visibility::Visibility,
};

use super::Engine;

impl Engine {
    pub fn tick_visibilities(&self) -> Vec<Message> {
        let mut messages = vec![];
        let tick_visibility =
            self.local_state.get_frame_i() % self.config.interiors_update_freq() == 0;

        if tick_visibility {
            messages.extend(self.update_visibilities())
        }

        messages
    }

    pub fn update_visibilities(&self) -> Vec<Message> {
        let mut visibilities = HashMap::new();
        let side_a_soldiers: Vec<&Soldier> = self
            .shared_state
            .soldiers()
            .iter()
            .filter(|s| s.get_side() == &Side::A)
            .collect();
        let side_b_soldiers: Vec<&Soldier> = self
            .shared_state
            .soldiers()
            .iter()
            .filter(|s| s.get_side() == &Side::B)
            .collect();

        for side_a_soldier in side_a_soldiers.iter().filter(|s| s.can_seek()) {
            for side_b_soldier in &side_b_soldiers {
                let visibility = Visibility::between_soldiers(
                    self.local_state.get_frame_i(),
                    side_a_soldier,
                    side_b_soldier,
                    &self.map,
                );
                visibilities.insert((side_a_soldier.uuid(), side_b_soldier.uuid()), visibility);
            }
        }

        for side_b_soldier in side_b_soldiers.into_iter().filter(|s| s.can_seek()) {
            for side_a_soldier in &side_a_soldiers {
                let visibility = Visibility::between_soldiers(
                    self.local_state.get_frame_i(),
                    side_b_soldier,
                    side_a_soldier,
                    &self.map,
                );
                visibilities.insert((side_b_soldier.uuid(), side_a_soldier.uuid()), visibility);
            }
        }

        vec![Message::LocalState(LocalStateMessage::SetVisibilities(
            visibilities,
        ))]
    }
}
