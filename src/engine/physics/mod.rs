use ggez::{graphics::MeshBuilder, GameResult};

use crate::{message::Message, NetworkMode};

use super::Engine;

mod bullet;
mod effects;
mod explosion;

impl Engine {
    pub fn tick_physics(&mut self) -> Vec<Message> {
        let mut messages = vec![];

        if self.local_state.get_frame_i() % self.config.physics_update_freq() == 0 {
            messages.extend(self.tick_bullet_fires());
            messages.extend(self.tick_explosions());

            messages.extend(self.fx_effects());
            if self.config.network_mode() == &NetworkMode::Server {
                messages.extend(self.resolve_effects())
            }
        }

        messages
    }

    pub fn draw_physics(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        self.draw_bullet_fires(mesh_builder)?;
        self.draw_explosions(mesh_builder)?;

        Ok(())
    }
}
