use glam::Vec2;

use crate::{behavior::Behavior, message::*, order::Order, types::*};

use super::Engine;

impl Engine {
    pub fn collect_player_inputs(&self) -> Vec<Message> {
        let mut messages = vec![];

        // TODO : hardcode code for test purposes
        if self.frame_i == 60 {
            for (i, entity) in self.state.entities().iter().enumerate() {
                if self.entity_is_squad_leader(EntityIndex(i)) {
                    match entity.get_behavior() {
                        Behavior::Idle => {
                            let a1 = entity.get_world_position().apply_raw(Vec2::new(10., 0.));
                            let a2 = entity.get_world_position().apply_raw(Vec2::new(0., 10.));
                            let b1 = entity.get_world_position().apply_raw(Vec2::new(20., 10.));
                            let b2 = entity.get_world_position().apply_raw(Vec2::new(10., 20.));
                            let a = WorldPath::new(vec![a1, a2]);
                            let b = WorldPath::new(vec![b1, b2]);
                            messages.push(Message::State(StateMessage::PushOrder(
                                entity.squad_uuid(),
                                Order::WalkTo(vec![a, b]),
                            )));
                        }
                        _ => {}
                    }
                }
            }
        }

        messages
    }
}
