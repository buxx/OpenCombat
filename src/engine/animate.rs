use crate::{message::*, types::*};

use super::Engine;

impl Engine {
    ///  - Compute visibility with other entities
    ///  - Compute behavior against physics (explosions, gunfires, ...)
    pub fn animate_entity(&self, i: EntityIndex) -> Vec<Message> {
        let entity = &self.state.entity(i);
        let mut messages = vec![];

        // FIXME demo code, this will be in "update" code
        let current_position = entity.get_world_position();
        let new_position = WorldPosition::from((
            current_position.x + WorldX::from(1.),
            current_position.y + WorldY::from(1.),
        ));
        let entity_message = EntityMessage::UpdateWorldPosition(new_position);
        let state_message = StateMessage::Entity(i, entity_message);
        let message = Message::State(state_message);
        messages.push(message);

        messages
    }
}
