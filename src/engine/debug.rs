use ggez::{
    graphics::{DrawMode, MeshBuilder},
    GameResult,
};

use crate::{
    behavior::Behavior,
    utils::{BLUE, YELLOW},
};

use super::Engine;

impl Engine {
    pub fn generate_debug_mouse_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        // Draw circle where left click down
        if let Some(point) = self.local_state.get_left_click_down_window_point() {
            mesh_builder.circle(DrawMode::fill(), point.to_vec2(), 2.0, 2.0, YELLOW)?;
        }

        // Draw circle at cursor position
        mesh_builder.circle(
            DrawMode::fill(),
            self.local_state.get_current_cursor_window_point().to_vec2(),
            2.0,
            2.0,
            BLUE,
        )?;

        Ok(())
    }

    pub fn generate_move_paths_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for (_, squad_composition) in self.shared_state.squads() {
            let squad_leader = self.shared_state.entity(squad_composition.leader());
            if let Some(world_paths) = match squad_leader.get_behavior() {
                Behavior::MoveTo(world_paths)
                | Behavior::MoveFastTo(world_paths)
                | Behavior::SneakTo(world_paths) => Some(world_paths),
                _ => None,
            } {
                for world_path in &world_paths.paths {
                    let last_point = self.local_state.window_point_from_world_point(
                        world_path.last_point().expect("Must contains point"),
                    );
                    mesh_builder.circle(
                        DrawMode::Fill(Default::default()),
                        last_point.to_vec2(),
                        5.0,
                        1.0,
                        YELLOW,
                    )?;

                    for point in &world_path.points {
                        mesh_builder.circle(
                            DrawMode::Fill(Default::default()),
                            self.local_state
                                .window_point_from_world_point(*point)
                                .to_vec2(),
                            2.0,
                            1.0,
                            BLUE,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}
