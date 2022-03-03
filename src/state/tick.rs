use crate::{message::*, types::*};
use rayon::prelude::*;

use super::MainState;

impl MainState {
    pub fn tick(&mut self) {
        // Will collect all tick messages
        let mut messages = vec![];

        // Entities computing
        if self.frame_i % self.config.entity_tick_freq() == 0 {
            let entity_messages: Vec<Message> = (0..self.entities.len())
                .into_par_iter()
                .flat_map(|i| self.tick_entity(i))
                .collect();
            messages.extend(entity_messages);
        }

        // Apply messages
        self.react(messages);
    }

    pub fn tick_entity(&self, i: EntityIndex) -> Vec<Message> {
        let mut messages = vec![];

        let entity = &self.entities[i];
        let current_position = entity.world_position();
        let new_position = WorldPosition::from((
            current_position.x + WorldX::from(1.),
            current_position.y + WorldY::from(1.),
        ));
        messages.push(Message::Entity(
            i,
            EntityMessage::UpdateWorldPosition(new_position),
        ));

        messages
    }
}
