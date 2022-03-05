use crate::message::Message;
use rayon::prelude::*;

use super::Engine;

impl Engine {
    pub fn tick_entities(&self) -> Vec<Message> {
        let mut messages = vec![];

        // Entities animation
        if self.frame_i % self.config.entity_animate_freq() == 0 {
            let entity_messages: Vec<Message> = (0..self.state.entities().len())
                .into_par_iter()
                .flat_map(|i| self.animate_entity(i))
                .collect();
            messages.extend(entity_messages);
        }

        // Entities updates
        if self.frame_i % self.config.entity_update_freq() == 0 {
            let entity_messages: Vec<Message> = (0..self.state.entities().len())
                .into_par_iter()
                .flat_map(|i| self.update_entity(i))
                .collect();
            messages.extend(entity_messages);
        }

        messages
    }
}
