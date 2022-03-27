use std::collections::HashMap;

use ggez::GameResult;

use crate::{message::*, order::Order, sync::StateCopy, types::*};

use super::SideEffect;

pub struct SharedState {
    /// Used to ignore server shared_state modifications since shared state not received from server
    initialized: bool,
    /// The entities in the world (soldiers, vehicles, etc).
    entities: Vec<ThreadSafeEntity>,
    /// Squad organizations, must be updated when squad leader changes.
    squads: HashMap<SquadUuid, SquadComposition>,
    /// Players orders. Squad leaders will pick from them theirs behaviors.
    command_orders: HashMap<SquadUuid, Order>,
    /// Squad leader orders. Squad members will pick from them theirs behaviors.
    squad_orders: HashMap<EntityIndex, Order>,
}

impl SharedState {
    pub fn new(entities: Vec<ThreadSafeEntity>) -> Self {
        Self {
            initialized: false,
            entities,
            squads: HashMap::new(),
            command_orders: HashMap::new(),
            squad_orders: HashMap::new(),
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

    pub fn command_orders(&self) -> &HashMap<SquadUuid, Order> {
        &self.command_orders
    }

    pub fn squad_orders(&self) -> &HashMap<EntityIndex, Order> {
        &self.squad_orders
    }

    pub fn all_orders(&self) -> Vec<(SquadUuid, Order)> {
        let mut orders: Vec<(SquadUuid, Order)> = vec![];

        for (squad_id, squad_composition) in &self.squads {
            let squad_leader = self.entity(squad_composition.leader());
            if let Some(order) = squad_leader.get_behavior().to_order() {
                orders.push((*squad_id, order));
            }
        }

        for (squad_id, order) in &self.command_orders {
            orders.push((*squad_id, order.clone()));
        }

        orders
    }

    pub fn squad(&self, squad_uuid: SquadUuid) -> &SquadComposition {
        self.squads
            .get(&squad_uuid)
            .expect("Game shared_state should never own inconsistent squad index")
    }

    pub fn react(&mut self, state_message: crate::message::SharedStateMessage) -> Vec<SideEffect> {
        match state_message {
            SharedStateMessage::Entity(entity_i, entity_message) => {
                return self.react_entity_message(entity_i, entity_message);
            }
            SharedStateMessage::PushCommandOrder(squad_uuid, order) => {
                self.command_orders.insert(squad_uuid, order);
            }
            SharedStateMessage::PushSquadOrder(entity_index, order) => {
                self.squad_orders.insert(entity_index, order);
            }
            SharedStateMessage::RemoveCommandOder(squad_uuid) => {
                self.command_orders
                    .remove(&squad_uuid)
                    .expect("Game shared_state should never own inconsistent orders index");
            }
            SharedStateMessage::RemoveSquadOder(entity_index) => {
                self.squad_orders
                    .remove(&entity_index)
                    .expect("Game shared_state should never own inconsistent orders index");
            }
        };

        vec![]
    }
}
