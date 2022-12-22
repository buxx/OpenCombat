use ggez::{
    graphics::{DrawMode, MeshBuilder},
    GameResult,
};

use crate::{
    behavior::Behavior,
    game::squad::{squad_positions, Formation},
    utils::{BLUE, DARK_MAGENTA, GREEN, MAGENTA, RED, YELLOW},
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
            let squad_leader = self.shared_state.soldier(squad_composition.leader());
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

    pub fn generate_formation_positions_meshes(
        &mut self,
        mesh_builder: &mut MeshBuilder,
    ) -> GameResult {
        // Display selected squad formation positions
        for squad_id in self.local_state.selected_squads() {
            let squad = self.shared_state.squad(*squad_id);
            for (_, point) in squad_positions(squad, Formation::Line, &self.shared_state) {
                let window_point = self.local_state.window_point_from_world_point(point);
                mesh_builder.circle(DrawMode::fill(), window_point.to_vec2(), 2.0, 2.0, YELLOW)?;
            }
        }

        Ok(())
    }

    pub fn generate_debug_point_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        let mut debug_points_left = vec![];
        while let Some(debug_point) = self.local_state.debug_points_mut().pop() {
            if debug_point.frame_i >= self.local_state.get_frame_i() {
                let window_point = self
                    .local_state
                    .window_point_from_world_point(debug_point.point);
                mesh_builder.circle(DrawMode::fill(), window_point.to_vec2(), 2.0, 2.0, BLUE)?;
                debug_points_left.push(debug_point);
            }
        }
        self.local_state.set_debug_points(debug_points_left);

        Ok(())
    }

    /// Draw circle on each soldier position
    pub fn generate_scene_item_circles_meshes(
        &mut self,
        mesh_builder: &mut MeshBuilder,
    ) -> GameResult {
        for soldier in self.shared_state.soldiers() {
            let color = if soldier.get_side() == self.local_state.side() {
                GREEN
            } else {
                RED
            };

            let point = self
                .local_state
                .window_point_from_world_point(soldier.get_world_point());
            mesh_builder.circle(DrawMode::fill(), point.to_vec2(), 2.0, 2.0, color)?;
        }

        Ok(())
    }

    /// Draw selection areas
    pub fn generate_areas_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        let cursor_world_point = self.local_state.get_current_cursor_world_point();
        let cursor_window_point = self.local_state.get_current_cursor_window_point();

        // Draw soldiers selection areas
        for soldier in self.shared_state.soldiers() {
            let rect = self
                .local_state
                .window_rect_from_world_rect(soldier.get_selection_rect());
            mesh_builder.rectangle(DrawMode::stroke(1.0), rect, MAGENTA)?;
        }

        // Draw selection area on cursor hover scene items
        for soldier_index in self.get_soldiers_at_point(cursor_world_point) {
            let soldier = self.shared_state.soldier(soldier_index);
            let rect = self
                .local_state
                .window_rect_from_world_rect(soldier.get_selection_rect());
            mesh_builder.rectangle(DrawMode::stroke(1.0), rect, DARK_MAGENTA)?;
        }

        // Draw selection area on all hovered soldiers
        for (_, order_marker, _, world_point, _) in self.shared_state.order_markers() {
            let rect = self.local_state.window_rect_from_world_rect(
                order_marker.sprite_info().get_selection_rect(world_point),
            );
            mesh_builder.rectangle(DrawMode::stroke(1.0), rect, MAGENTA)?;
            if order_marker.sprite_info().contains(
                &self.local_state.window_point_from_world_point(world_point),
                &cursor_window_point,
            ) {
                mesh_builder.rectangle(DrawMode::stroke(1.0), rect, DARK_MAGENTA)?;
            }
        }

        Ok(())
    }
}
