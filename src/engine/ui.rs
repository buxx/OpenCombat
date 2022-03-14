use ggez::{
    graphics::{DrawMode, MeshBuilder, Rect},
    GameResult,
};

use crate::utils::GREEN;

use super::Engine;

impl Engine {
    pub fn generate_select_rectangle_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if let Some((start, end)) = self.local_state.current_cursor_vector_world_points() {
            mesh_builder.rectangle(
                DrawMode::stroke(1.0),
                Rect::new(start.x, start.y, end.x - start.x, end.y - start.y),
                GREEN,
            )?;
        }

        Ok(())
    }
}
