use crate::{
    behavior::BehaviorMode,
    entity::soldier::Soldier,
    game::squad::{squad_positions, Formation},
    map::util::find_cover_grid_point,
    message::*,
    order::Order,
    physics::path::{find_path, PathMode},
    types::*,
    utils::DebugPoint,
};
use rayon::prelude::*;

use super::Engine;

impl Engine {
    // FIXME BS NOW : Soldiers in vehicles must be managed differently than ground soldiers
    pub fn tick_soldiers(&self) -> Vec<Message> {
        let mut messages = vec![];
        let tick_animate = self.local_state.get_frame_i() % self.config.soldier_animate_freq() == 0;
        let tick_update = self.local_state.get_frame_i() % self.config.soldier_update_freq() == 0;

        // Entities animation
        if tick_animate {
            messages.extend(
                (0..self.shared_state.soldiers().len())
                    .into_par_iter()
                    .flat_map(|i| self.animate_soldier(SoldierIndex(i)))
                    .collect::<Vec<Message>>(),
            );
        }

        // Entities updates
        if tick_update {
            let soldier_messages: Vec<Message> = (0..self.shared_state.soldiers().len())
                .into_par_iter()
                .flat_map(|i| self.update_soldier(SoldierIndex(i)))
                .collect();
            messages.extend(soldier_messages);
        }

        messages
    }

    pub fn tick_feeling_decreasing_soldiers(&self) -> Vec<Message> {
        let mut messages = vec![];
        let tick_feeling_decreasing =
            self.local_state.get_frame_i() % self.config.feeling_decreasing_freq() == 0;

        if tick_feeling_decreasing {
            let soldier_messages: Vec<Message> = (0..self.shared_state.soldiers().len())
                .into_par_iter()
                .flat_map(|i| self.decrease_feeling(SoldierIndex(i)))
                .collect();
            messages.extend(soldier_messages);
        }

        messages
    }

    pub fn soldier_is_squad_leader(&self, soldier_index: SoldierIndex) -> bool {
        let soldier = self.shared_state.soldier(soldier_index);
        let squad_uuid = soldier.squad_uuid();
        let squad_composition = self.shared_state.squad(squad_uuid);
        let squad_leader = squad_composition.leader();
        squad_leader == soldier_index
    }
}
