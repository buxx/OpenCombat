use crate::{
    behavior::feeling::Feeling,
    message::{Message, SharedStateMessage, SoldierMessage},
    types::{Meters, SoldierIndex},
};

use super::Engine;

impl Engine {
    // TODO : have a real algorithm here
    pub fn soldier_bullet_injured(&self, _soldier_index: SoldierIndex) -> Vec<Message> {
        vec![]
    }

    // TODO : have a real algorithm here
    pub fn soldier_proximity_bullet(
        &self,
        soldier_index: SoldierIndex,
        distance: Meters,
    ) -> Vec<Message> {
        vec![Message::SharedState(SharedStateMessage::Soldier(
            soldier_index,
            SoldierMessage::IncreaseUnderFire(
                Feeling::UnderFire(0).proximity_bullet_increase_value(distance),
            ),
        ))]
    }
}
