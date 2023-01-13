use crate::{engine::Engine, message::Message, types::SoldierIndex};

impl Engine {
    pub fn engage_soldier_update(
        &self,
        solider_index: &SoldierIndex,
        opponent_index: &SoldierIndex,
    ) -> Vec<Message> {
        todo!()
    }
}
