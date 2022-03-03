use crate::{entity::Entity, message::EntityMessage};

pub struct State {
    entities: Vec<Box<dyn Entity + Send + Sync>>,
}

impl State {
    pub fn new(entities: Vec<Box<dyn Entity + Send + Sync>>) -> Self {
        Self { entities }
    }

    pub fn entities(&self) -> &Vec<Box<dyn Entity + Send + Sync>> {
        &self.entities
    }

    pub fn entity(&self, entity_index: usize) -> &Box<dyn Entity + Send + Sync> {
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
