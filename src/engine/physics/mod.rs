use ggez::{graphics::MeshBuilder, GameResult};

use crate::message::Message;

use super::Engine;

mod bullet;
mod explosion;

impl Engine {
    pub fn tick_physics(&self) -> Vec<Message> {
        let mut messages = vec![];

        if self.local_state.get_frame_i() % self.config.physics_update_freq() == 0 {
            messages.extend(self.tick_bullet_fires());
            messages.extend(self.tick_explosions());
        }

        messages
    }

    pub fn draw_physics(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        self.draw_bullet_fires(mesh_builder)?;
        self.draw_explosions(mesh_builder)?;

        Ok(())
    }
}
