use crate::{message::*, sync::StateCopy};

use super::Engine;

impl Engine {
    pub fn react(&mut self, messages: Vec<Message>) {
        // Dispatch messages depend on network mode
        match self.config.network_mode() {
            crate::NetWorkMode::Server => self.dispatch_as_server(&messages),
            crate::NetWorkMode::Client => self.dispatch_as_client(&messages),
        }

        for message in messages {
            match message {
                Message::State(state_message) => match state_message {
                    StateMessage::Entity(entity_i, entity_message) => {
                        self.shared_state
                            .react_entity_message(entity_i, entity_message);
                    }
                    StateMessage::PushOrder(squad_uuid, order) => {
                        self.shared_state.push_order(squad_uuid, order);
                    }
                    StateMessage::RemoveOder(squad_uuid) => {
                        self.shared_state.remove_order(squad_uuid);
                    }
                },
                Message::Network(network_message) => match network_message {
                    NetworkMessage::RequireCompleteSync => {
                        self.send_complete_sync();
                    }
                    NetworkMessage::InitializeStateFrom(state_copy) => {
                        self.shared_state.init_from_copy(state_copy);
                    }
                    NetworkMessage::Acknowledge => unreachable!(),
                },
                Message::Engine(engine_message) => match engine_message {
                    EngineMessage::ApplySceneDisplayOffset(offset) => {
                        self.display_scene_offset += offset.to_vec2();
                    }
                },
            }
        }
    }

    fn send_complete_sync(&self) {
        let state_copy = StateCopy::from_state(&self.shared_state);
        let network_message = NetworkMessage::InitializeStateFrom(state_copy);
        let message = Message::Network(network_message);
        self.network.send(vec![message]);
    }
}
