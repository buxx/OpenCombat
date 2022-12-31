use ggez::graphics::MeshBuilder;
use ggez::GameResult;

use crate::engine::Engine;

use crate::{
    message::{LocalStateMessage, Message},
    physics::event::explosion::Explosion,
    types::ExplosionIndex,
    NetworkMode,
};

impl Engine {
    pub fn tick_explosions(&self) -> Vec<Message> {
        let mut messages = vec![];
        let frame_i = self.local_state.get_frame_i();

        for (i, explosion) in self.local_state.explosions().iter().enumerate() {
            messages.extend(explosion.fx(self.local_state.get_frame_i()));

            if self.config.network_mode() == &NetworkMode::Server && explosion.effective(frame_i) {
                messages.extend(self.explosion_effects(explosion))
            }

            if explosion.finished(frame_i) {
                messages.push(Message::LocalState(LocalStateMessage::RemoveExplosion(
                    ExplosionIndex(i),
                )))
            }
        }

        messages
    }

    fn explosion_effects(&self, explosion: &Explosion) -> Vec<Message> {
        let mut messages = vec![];

        messages
    }

    pub fn draw_explosions(&self, _mesh_builder: &mut MeshBuilder) -> GameResult {
        // Nothing here because drawn by graphics sequences
        Ok(())
    }
}
