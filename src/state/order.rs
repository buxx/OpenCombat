use crate::{order::Order, types::EntityIndex};

use super::State;

impl State {
    pub fn order_for(&self, entity_index: EntityIndex) -> Option<&Order> {
        let entity = self.entity(entity_index);
        self.orders.get(&entity.squad_uuid())
    }
}