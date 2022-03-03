use crate::{message::EntityMessage, types::*};

pub struct State {
    entities: Vec<ThreadSafeEntity>,
}

impl State {
    pub fn new(entities: Vec<ThreadSafeEntity>) -> Self {
        Self { entities }
    }

    pub fn entities(&self) -> &Vec<ThreadSafeEntity> {
        &self.entities
    }

    pub fn entity(&self, entity_index: usize) -> &ThreadSafeEntity {
        &self.entities[entity_index]
    }

    pub fn react_entity_message(&mut self, entity_i: usize, entity_message: EntityMessage) {
        let entity = &mut self.entities[entity_i];
        match entity_message {
            EntityMessage::UpdateWorldPosition(new_world_position) => {
                entity.set_world_position(new_world_position)
            }
        }
    }
}
