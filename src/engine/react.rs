use ggez::{Context, GameResult};

use crate::{
    message::{Message, NetworkMessage, PhysicsMessage, SideEffect},
    sync::StateCopy,
};

use super::Engine;

impl Engine {
    pub fn react(
        &mut self,
        messages: Vec<Message>,
        ctx: &mut Context,
    ) -> GameResult<Vec<SideEffect>> {
        let mut side_effects = vec![];

        for message in messages {
            match message {
                Message::SharedState(shared_state_message) => {
                    side_effects.extend(self.shared_state.react(shared_state_message));
                }
                Message::LocalState(local_state_message) => {
                    self.local_state.react(local_state_message)
                }
                Message::Network(network_message) => match network_message {
                    NetworkMessage::RequireCompleteSync => {
                        self.send_complete_sync();
                    }
                    NetworkMessage::InitializeStateFrom(state_copy) => {
                        self.shared_state.init_from_copy(state_copy);
                        self.graphics.initialize(self.shared_state.soldiers());
                    }
                    NetworkMessage::Acknowledge => unreachable!(),
                },
                Message::Graphics(graphics_message) => {
                    //
                    self.graphics
                        .react(graphics_message, &self.map, &self.config, ctx)?;
                }
                Message::Physics(physics_message) => match physics_message {
                    PhysicsMessage::PushBulletFire(mut bullet_fire) => {
                        // Add one to current frame because bullet fire will begin to be process at the next frame
                        bullet_fire.init(self.local_state.get_frame_i() + 1);
                        self.local_state.push_bullet_fire(bullet_fire)
                    }
                    PhysicsMessage::PushExplosion(mut explosion) => {
                        // Add one to current frame because explosion will begin to be process at the next frame
                        explosion.init(self.local_state.get_frame_i() + 1);
                        self.local_state.push_explosion(explosion)
                    }
                },
            }
        }

        GameResult::Ok(side_effects)
    }

    fn send_complete_sync(&self) {
        let state_copy = StateCopy::from_state(&self.shared_state);
        let network_message = NetworkMessage::InitializeStateFrom(state_copy);
        let message = Message::Network(network_message);
        self.network.send(vec![message]);
    }
}
