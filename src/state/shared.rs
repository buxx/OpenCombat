use std::collections::HashMap;

use ggez::GameResult;

use crate::{entity::soldier::Soldier, message::*, order::Order, sync::StateCopy, types::*};

use super::SideEffect;

pub struct SharedState {
    /// Used to ignore server shared_state modifications since shared state not received from server
    initialized: bool,
    /// The soldiers in the world (soldiers, vehicles, etc).
    soldiers: Vec<Soldier>,
    /// Squad organizations, must be updated when squad leader changes.
    squads: HashMap<SquadUuid, SquadComposition>,
    /// Players orders. Squad leaders will pick from them theirs behaviors.
    command_orders: HashMap<SquadUuid, Order>,
    /// Squad leader orders. Squad members will pick from them theirs behaviors.
    squad_orders: HashMap<SoldierIndex, Order>,
}

impl SharedState {
    pub fn new(soldiers: Vec<Soldier>) -> Self {
        Self {
            initialized: false,
            soldiers: soldiers,
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
        self.soldiers = vec![];

        for soldier in state_copy.soldiers() {
            self.soldiers.push(soldier.clone());
        }

        // TODO
        self.init().unwrap();
    }

    pub fn initialized(&self) -> bool {
        self.initialized
    }

    pub fn soldiers(&self) -> &Vec<Soldier> {
        &self.soldiers
    }

    pub fn soldier(&self, soldier_index: SoldierIndex) -> &Soldier {
        &self.soldiers[soldier_index.0]
    }

    pub fn soldier_mut(&mut self, soldier_index: SoldierIndex) -> &mut Soldier {
        &mut self.soldiers[soldier_index.0]
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

    pub fn squad_orders(&self) -> &HashMap<SoldierIndex, Order> {
        &self.squad_orders
    }

    pub fn all_orders(&self) -> Vec<(SquadUuid, Order)> {
        let mut orders: Vec<(SquadUuid, Order)> = vec![];

        for (squad_id, squad_composition) in &self.squads {
            let squad_leader = self.soldier(squad_composition.leader());
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
            SharedStateMessage::Entity(soldier_index, soldier_message) => {
                return self.react_soldier_message(soldier_index, soldier_message);
            }
            SharedStateMessage::PushCommandOrder(squad_uuid, order) => {
                self.command_orders.insert(squad_uuid, order);
            }
            SharedStateMessage::PushSquadOrder(soldier_index, order) => {
                self.squad_orders.insert(soldier_index, order);
            }
            SharedStateMessage::RemoveCommandOder(squad_uuid) => {
                self.command_orders
                    .remove(&squad_uuid)
                    .expect("Game shared_state should never own inconsistent orders index");
            }
            SharedStateMessage::RemoveSquadOder(soldier_index) => {
                self.squad_orders
                    .remove(&soldier_index)
                    .expect("Game shared_state should never own inconsistent orders index");
            }
        };

        vec![]
    }
}
