use ggez::{graphics::MeshBuilder, GameResult};

use crate::message::Message;

use super::Engine;

impl Engine {
    pub fn tick_physics(&mut self) -> Vec<Message> {
        self.shared_state
            .physics_mut()
            .tick(self.local_state.get_frame_i())
    }

    pub fn draw_physics(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        self.shared_state
            .physics()
            .draw(mesh_builder, self.local_state.get_frame_i())?;

        Ok(())
    }
}
