use std::collections::HashMap;

use ggez::GameResult;

use crate::{message::EntityMessage, order::Order, sync::StateCopy, types::*};

mod squad;

pub struct State {
    /// Used to ignore server state modifications since state not received from server
    initialized: bool,
    /// The entities in the world (soldiers, vehicles, etc).
    entities: Vec<ThreadSafeEntity>,
    /// Squad organizations, must be updated when squad leader changes.
    squads: Squads,
    /// Players orders. Entities will pick from them theirs behaviors.
    /// // FIXME : This should be in Engine instead State ? (because Sate shared to clients)
    _orders: HashMap<SquadIndex, Vec<Order>>,
}

impl State {
    pub fn new(entities: Vec<ThreadSafeEntity>) -> Self {
        Self {
            initialized: false,
            entities,
            squads: vec![],
            _orders: HashMap::new(),
        }
    }

    pub fn init(&mut self) -> GameResult {
        // At start point, squads have not been defined. We must initialize it.
        self.update_squads();
        self.initialized = true;
        Ok(())
    }

    pub fn init_from_copy(&mut self, state_copy: StateCopy) {
        self.entities = vec![];

        for soldier in state_copy.soldiers() {
            self.entities.push(Box::new(soldier.clone()));
        }

        // TODO
        self.init().unwrap();
    }

    pub fn entities(&self) -> &Vec<ThreadSafeEntity> {
        &self.entities
    }

    pub fn entity(&self, entity_index: usize) -> &ThreadSafeEntity {
        &self.entities[entity_index]
    }

    pub fn react_entity_message(&mut self, entity_i: usize, entity_message: EntityMessage) {
        if !self.initialized {
            return;
        }

        let entity = &mut self.entities[entity_i];
        match entity_message {
            EntityMessage::SetWorldPosition(new_world_position) => {
                entity.set_world_position(new_world_position)
            }
            EntityMessage::SetBehavior(behavior) => entity.set_behavior(behavior),
        }
    }
}
