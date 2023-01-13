use ggez::{
    graphics::{Color, DrawMode, MeshBuilder},
    GameResult,
};

use crate::{
    behavior::Behavior,
    debug::DebugPhysics,
    game::{
        explosive::Type as ExplosiveType,
        squad::{squad_positions, Formation},
        weapon::Weapon,
        Side,
    },
    message::{Message, PhysicsMessage},
    physics::event::{bullet::BulletFire, explosion::Explosion},
    types::WorldPoint,
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
            if let Some(world_paths) = match squad_leader.behavior() {
                Behavior::MoveTo(world_paths)
                | Behavior::MoveFastTo(world_paths)
                | Behavior::SneakTo(world_paths)
                | Behavior::DriveTo(world_paths) => Some(world_paths),

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

        // Draw vehicle physics areas
        for vehicle in self.shared_state.vehicles() {
            let shape = vehicle
                .get_chassis_shape()
                .to_window_shape(&self.local_state);

            mesh_builder.line(&shape.draw_points(), 1.0, MAGENTA)?;
        }

        // Draw selection area on cursor hover scene items
        for soldier_index in self.get_soldiers_at_point(cursor_world_point) {
            let soldier = self.shared_state.soldier(soldier_index);
            let rect = self
                .local_state
                .window_rect_from_world_rect(soldier.get_selection_rect());
            mesh_builder.rectangle(DrawMode::stroke(1.0), rect, DARK_MAGENTA)?;
        }

        // Draw selection area on all order markers
        for (order, order_marker, _, world_point, _) in self.shared_state.order_markers(&Side::All)
        {
            let shape = self
                .defend_order_selection_shape(&order, &order_marker, &world_point)
                .to_window_shape(&self.local_state);
            let color = if shape.contains(cursor_window_point) {
                DARK_MAGENTA
            } else {
                MAGENTA
            };
            mesh_builder.line(&shape.draw_points(), 1.0, color)?;
        }

        Ok(())
    }

    /// Draw selection areas
    pub fn generate_visibilities_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for squad_uuid in self.local_state.selected_squads() {
            let squad_composition = self.shared_state.squad(*squad_uuid);
            for soldier_index in squad_composition.members() {
                let from_soldier = self.shared_state.soldier(*soldier_index);
                let to_soldiers = self
                    .shared_state
                    .soldiers()
                    .iter()
                    .filter(|s| s.get_side() != from_soldier.get_side());
                for to_soldier in to_soldiers {
                    if let Some(visibility) = self
                        .local_state
                        .visibilities()
                        .get(&(from_soldier.uuid(), to_soldier.uuid()))
                    {
                        let start_world_point = from_soldier.get_world_point();
                        let mut previous_point = self
                            .local_state
                            .window_point_from_world_point(start_world_point);
                        let mut previous_opacity: f32 = 0.0;

                        for (segment_world_point, segment_new_opacity) in
                            visibility.opacity_segments.iter().skip(1)
                        {
                            let segment_point = self
                                .local_state
                                .window_point_from_world_point(*segment_world_point);
                            let mut color_canal_value = 1.0 - previous_opacity;
                            if color_canal_value < 0.0 {
                                color_canal_value = 0.0;
                            }
                            mesh_builder.line(
                                &vec![previous_point.to_vec2(), segment_point.to_vec2()],
                                1.0,
                                Color {
                                    r: color_canal_value,
                                    g: color_canal_value,
                                    b: color_canal_value,
                                    a: 1.0,
                                },
                            )?;

                            previous_point = segment_point;
                            previous_opacity = *segment_new_opacity;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn generate_debug_physics(&self, from: WorldPoint, to: WorldPoint) -> Vec<Message> {
        let mut messages = vec![];

        match self.local_state.get_debug_physics() {
            DebugPhysics::None => {}
            DebugPhysics::MosinNagantM1924GunFire => {
                messages.push(Message::Physics(PhysicsMessage::PushBulletFire(
                    BulletFire::new(from, to, None, Weapon::MosinNagantM1924),
                )));
            }
            DebugPhysics::BrandtMle2731Shelling => {
                messages.push(Message::Physics(PhysicsMessage::PushExplosion(
                    Explosion::new(from, ExplosiveType::FA19241927),
                )));
            }
        };

        messages
    }
}
