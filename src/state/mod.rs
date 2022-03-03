use std::collections::HashMap;

use crate::{message::EntityMessage, order::Order, types::*};

mod squad;

pub struct State {
    /// The entities in the world (soldiers, vehicles, etc).
    entities: Vec<ThreadSafeEntity>,
    /// Squad organizations, must be updated when squad leader changes.
    squads: Squads,
    /// Players orders. Entities will pick from them theirs behaviors.
    _orders: HashMap<SquadIndex, Vec<Order>>,
}

impl State {
    pub fn new(entities: Vec<ThreadSafeEntity>) -> Self {
        Self {
            entities,
            squads: vec![],
            _orders: HashMap::new(),
        }
    }

    pub fn initialize(&mut self) {
        // At start point, squads have not been defined. We must initialize it.
        self.update_squads();
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
