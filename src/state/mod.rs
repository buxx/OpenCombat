use std::collections::HashMap;

use ggez::GameResult;

use crate::{message::EntityMessage, order::Order, sync::StateCopy, types::*};

mod order;
mod squad;

pub struct State {
    /// Used to ignore server state modifications since state not received from server
    initialized: bool,
    /// The entities in the world (soldiers, vehicles, etc).
    entities: Vec<ThreadSafeEntity>,
    /// Squad organizations, must be updated when squad leader changes.
    squads: HashMap<SquadUuid, SquadComposition>,
    /// Players orders. Entities will pick from them theirs behaviors.
    /// // FIXME : This should be in Engine instead State ? (because Sate shared to clients) Maybe yes, in state
    orders: HashMap<SquadUuid, Order>,
}

impl State {
    pub fn new(entities: Vec<ThreadSafeEntity>) -> Self {
        Self {
            initialized: false,
            entities,
            squads: HashMap::new(),
            orders: HashMap::new(),
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

    pub fn entity(&self, entity_index: EntityIndex) -> &ThreadSafeEntity {
        &self.entities[entity_index.0]
    }

    pub fn react_entity_message(&mut self, entity_i: EntityIndex, entity_message: EntityMessage) {
        if !self.initialized {
            return;
        }

        let entity = &mut self.entities[entity_i.0];
        match entity_message {
            EntityMessage::SetWorldPosition(new_world_point) => {
                entity.set_world_point(new_world_point)
            }
            EntityMessage::SetBehavior(behavior) => entity.set_behavior(behavior),
            EntityMessage::SetOrientation(angle) => entity.set_looking_direction(angle),
            EntityMessage::ReachBehaviorStep => entity.get_behavior_mut().reach_step(),
        }
    }

    pub fn squad(&self, squad_uuid: SquadUuid) -> &SquadComposition {
        self.squads
            .get(&squad_uuid)
            .expect("Game state should never own inconsistent squad index")
    }

    pub fn _squads(&self) -> &HashMap<SquadUuid, SquadComposition> {
        &self.squads
    }

    pub fn _orders(&self) -> &HashMap<SquadUuid, Order> {
        &self.orders
    }

    pub fn push_order(&mut self, squad_uuid: SquadUuid, order: Order) {
        self.orders.insert(squad_uuid, order);
    }

    pub fn remove_order(&mut self, squad_uuid: SquadUuid) {
        self.orders
            .remove(&squad_uuid)
            .expect("Game state should never own inconsistent orders index");
    }
}
