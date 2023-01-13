use super::Engine;
use crate::{message::Message, types::SoldierIndex};

impl Engine {
    ///  - Compute visibility with other soldiers
    ///  - Compute behavior against physics (explosions, gunfires, ...)
    pub fn animate_soldier(&self, soldier_index: SoldierIndex) -> Vec<Message> {
        let soldier = self.shared_state.soldier(soldier_index);
        if !soldier.can_be_animated() {
            return vec![];
        }

        let mut messages = vec![];

        messages.extend(self.soldier_behavior(soldier));
        messages.extend(self.soldier_gesture(soldier));

        messages
    }
}
