use crate::{
    behavior::{walking, Behavior},
    message::*,
    types::*,
    utils::angle,
};

use super::Engine;

impl Engine {
    ///  - World pixel point according to movement
    ///  - ...
    pub fn update_entity(&self, i: EntityIndex) -> Vec<Message> {
        let entity = self.state.entity(i);
        let mut messages = vec![];

        messages.extend(self.orientation_update(i));

        let entity_messages = match entity.get_behavior() {
            Behavior::Idle => {
                vec![]
            }
            Behavior::MoveTo(paths) => walking::entity_updates(entity, paths),
        };
        messages.extend(Message::vec_from_entity(i, entity_messages));

        messages
    }

    fn orientation_update(&self, i: EntityIndex) -> Vec<Message> {
        let entity = self.state.entity(i);
        let mut messages = vec![];

        if let Some(point) = entity.get_behavior().looking_point() {
            let orientation = angle(&point, &entity.get_world_point());
            let entity_message = EntityMessage::SetOrientation(orientation);
            messages.push(Message::State(StateMessage::Entity(i, entity_message)));
        }

        messages
    }
}
