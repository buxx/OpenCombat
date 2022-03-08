use glam::Vec2;

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
                let a1 = entity.get_world_position().apply_raw(Vec2::new(10., 0.));
                let a2 = entity.get_world_position().apply_raw(Vec2::new(0., 10.));
                let b1 = entity.get_world_position().apply_raw(Vec2::new(20., 10.));
                let b2 = entity.get_world_position().apply_raw(Vec2::new(10., 20.));
                let a = WorldPath::new(vec![a1, a2]);
                let b = WorldPath::new(vec![b1, b2]);
                messages.push(Message::State(StateMessage::Entity(
                    i,
                    EntityMessage::SetBehavior(Behavior::WalkingTo(vec![a, b])),
                )));
            }
            crate::behavior::Behavior::WalkingTo(_) => {}
        }

        messages
    }
}
