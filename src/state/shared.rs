use std::collections::HashMap;

use ggez::GameResult;

use crate::{message::*, order::Order, sync::StateCopy, types::*};

pub struct SharedState {
    /// Used to ignore server shared_state modifications since shared state not received from server
    initialized: bool,
    /// The entities in the world (soldiers, vehicles, etc).
    entities: Vec<ThreadSafeEntity>,
    /// Squad organizations, must be updated when squad leader changes.
    squads: HashMap<SquadUuid, SquadComposition>,
    /// Players orders. Entities will pick from them theirs behaviors.
    /// // FIXME : This should be in Engine instead State ? (because Sate shared to clients) Maybe yes, in state
    given_orders: HashMap<SquadUuid, Order>,
}

impl SharedState {
    pub fn new(entities: Vec<ThreadSafeEntity>) -> Self {
        Self {
            initialized: false,
            entities,
            squads: HashMap::new(),
            given_orders: HashMap::new(),
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

    pub fn initialized(&self) -> bool {
        self.initialized
    }

    pub fn entities(&self) -> &Vec<ThreadSafeEntity> {
        &self.entities
    }

    pub fn entity(&self, entity_index: EntityIndex) -> &ThreadSafeEntity {
        &self.entities[entity_index.0]
    }

    pub fn entity_mut(&mut self, entity_index: EntityIndex) -> &mut ThreadSafeEntity {
        &mut self.entities[entity_index.0]
    }

    pub fn squads(&self) -> &HashMap<SquadUuid, SquadComposition> {
        &self.squads
    }

    pub fn set_squads(&mut self, squads: HashMap<SquadUuid, SquadComposition>) {
        self.squads = squads;
    }

    pub fn given_orders(&self) -> &HashMap<SquadUuid, Order> {
        &self.given_orders
    }

    pub fn all_orders(&self) -> Vec<(SquadUuid, Order)> {
        let mut orders: Vec<(SquadUuid, Order)> = vec![];

        for (squad_id, squad_composition) in &self.squads {
            let squad_leader = self.entity(squad_composition.leader());
            if let Some(order) = squad_leader.get_behavior().as_order() {
                orders.push((*squad_id, order));
            }
        }

        for (squad_id, order) in &self.given_orders {
            orders.push((*squad_id, order.clone()));
        }

        orders
    }

    pub fn squad(&self, squad_uuid: SquadUuid) -> &SquadComposition {
        self.squads
            .get(&squad_uuid)
            .expect("Game shared_state should never own inconsistent squad index")
    }

    pub fn react(&mut self, state_message: crate::message::SharedStateMessage) {
        match state_message {
            SharedStateMessage::Entity(entity_i, entity_message) => {
                self.react_entity_message(entity_i, entity_message);
            }
            SharedStateMessage::PushGivenOrder(squad_uuid, order) => {
                self.given_orders.insert(squad_uuid, order);
            }
            SharedStateMessage::RemoveOder(squad_uuid) => {
                self.given_orders
                    .remove(&squad_uuid)
                    .expect("Game shared_state should never own inconsistent orders index");
            }
        }
    }
}
