use rayon::prelude::*;
use std::collections::HashMap;

use crate::{
    entity::soldier::Soldier,
    game::Side,
    message::{LocalStateMessage, Message},
    physics::visibility::Visibility,
    types::SoldierIndex,
};

use super::Engine;

impl Engine {
    pub fn tick_visibilities(&self) -> Vec<Message> {
        let mut messages = vec![];
        let tick_visibility =
            self.local_state.get_frame_i() % self.config.visibility_update_freq() == 0;

        if tick_visibility {
            messages.extend(self.update_visibilities())
        }

        messages
    }

    pub fn update_visibilities(&self) -> Vec<Message> {
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

        let from_side_a_visibilities: HashMap<(SoldierIndex, SoldierIndex), Visibility> =
            side_a_soldiers
                .iter()
                .map(|s| s.uuid())
                .collect::<Vec<SoldierIndex>>()
                .into_par_iter()
                .flat_map(|i| self.soldier_visibilities(i, &side_b_soldiers))
                .collect();
        let from_side_b_visibilities: HashMap<(SoldierIndex, SoldierIndex), Visibility> =
            side_b_soldiers
                .iter()
                .map(|s| s.uuid())
                .collect::<Vec<SoldierIndex>>()
                .into_par_iter()
                .flat_map(|i| self.soldier_visibilities(i, &side_a_soldiers))
                .collect();

        vec![Message::LocalState(LocalStateMessage::SetVisibilities(
            from_side_a_visibilities
                .into_iter()
                .chain(from_side_b_visibilities)
                .collect(),
        ))]
    }

    pub fn soldier_visibilities(
        &self,
        soldier_index: SoldierIndex,
        other_soldiers: &Vec<&Soldier>,
    ) -> HashMap<(SoldierIndex, SoldierIndex), Visibility> {
        let mut visibilities = HashMap::new();
        let soldier = self.shared_state.soldier(soldier_index);

        if !soldier.can_seek() {
            return visibilities;
        }

        for other_soldier in other_soldiers {
            visibilities.insert(
                (soldier.uuid(), other_soldier.uuid()),
                Visibility::between_soldiers(
                    self.local_state.get_frame_i(),
                    soldier,
                    other_soldier,
                    &self.map,
                ),
            );
        }

        visibilities
    }
}
