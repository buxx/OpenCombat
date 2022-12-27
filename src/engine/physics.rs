use ggez::{graphics::MeshBuilder, GameResult};

use crate::message::Message;

use super::Engine;

impl Engine {
    pub fn tick_physics(&mut self) -> Vec<Message> {
        if self.local_state.get_frame_i() % self.config.physics_update_freq() == 0 {
            self.shared_state
                .physics_mut()
                .tick(self.local_state.get_frame_i())
        } else {
            vec![]
        }
    }

    pub fn draw_physics(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        self.shared_state
            .physics()
            .draw(mesh_builder, &self.local_state)?;

        Ok(())
    }
}
