use crate::{behavior::Behavior, message::*, types::*};

use super::Engine;

impl Engine {
    ///  - Compute visibility with other entities
    ///  - Compute behavior against physics (explosions, gunfires, ...)
    pub fn animate_entity(&self, i: EntityIndex) -> Vec<Message> {
        let entity = self.state.entity(i);
        let mut messages = vec![];

        // FIXME tmp code dor dev
        match entity.get_behavior() {
            Behavior::Idle => {
                messages.push(Message::State(StateMessage::Entity(
                    i,
                    EntityMessage::SetBehavior(Behavior::WalkingTo(WorldPosition::from((
                        WorldX::from(250.),
                        WorldY::from(250.),
                    )))),
                )));
            }
            crate::behavior::Behavior::WalkingTo(_) => {}
        }

        messages
    }
}
