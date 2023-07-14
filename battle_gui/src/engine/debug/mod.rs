use ggez::{
    graphics::{Color, DrawMode, MeshBuilder},
    GameResult,
};

use battle_core::{
    behavior::Behavior,
    game::{
        explosive::ExplosiveType,
        squad::{squad_positions, Formation},
        weapon::Weapon,
        Side,
    },
    physics::event::{bullet::BulletFire, explosion::Explosion},
    state::battle::message::BattleStateMessage,
    types::WorldPoint,
};

use crate::{
    debug::DebugPhysics,
    utils::{BLUE, DARK_MAGENTA, GREEN, MAGENTA, RED, YELLOW},
};

use super::{message::EngineMessage, Engine};
pub mod gui;

impl Engine {
    pub fn generate_debug_mouse_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        // Draw circle where left click down
        if let Some(point) = self.gui_state.left_click_down_window_point() {
            mesh_builder.circle(DrawMode::fill(), point.to_vec2(), 2.0, 2.0, YELLOW)?;
        }

        // Draw circle at cursor position
        mesh_builder.circle(
            DrawMode::fill(),
            self.gui_state.current_cursor_window_point().to_vec2(),
            2.0,
            2.0,
            BLUE,
        )?;

        Ok(())
    }

    pub fn generate_move_paths_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for (_, squad_composition) in self.battle_state.squads() {
            let squad_leader = self.battle_state.soldier(squad_composition.leader());
            if let Some(world_paths) = match squad_leader.behavior() {
                Behavior::MoveTo(world_paths)
                | Behavior::MoveFastTo(world_paths)
                | Behavior::SneakTo(world_paths)
                | Behavior::DriveTo(world_paths) => Some(world_paths),

                _ => None,
            } {
                for world_path in &world_paths.paths {
                    let last_point = self.gui_state.window_point_from_world_point(
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
                            self.gui_state
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
        for squad_id in &self.gui_state.selected_squads().1 {
            let squad = self.battle_state.squad(*squad_id);
            let leader = self.battle_state.soldier(squad.leader());
            for (_, point) in squad_positions(squad, Formation::Line, leader, None) {
                let window_point = self.gui_state.window_point_from_world_point(point);
                mesh_builder.circle(DrawMode::fill(), window_point.to_vec2(), 2.0, 2.0, YELLOW)?;
            }
        }

        Ok(())
    }

    pub fn generate_debug_point_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        let mut debug_points_left = vec![];
        while let Some(debug_point) = self.gui_state.debug_points_mut().pop() {
            if debug_point.frame_i >= self.gui_state.frame_i() {
                let window_point = self
                    .gui_state
                    .window_point_from_world_point(debug_point.point);
                mesh_builder.circle(DrawMode::fill(), window_point.to_vec2(), 2.0, 2.0, BLUE)?;
                debug_points_left.push(debug_point);
            }
        }
        self.gui_state.set_debug_points(debug_points_left);

        Ok(())
    }

    /// Draw circle on each soldier position
    pub fn generate_scene_item_circles_meshes(
        &mut self,
        mesh_builder: &mut MeshBuilder,
    ) -> GameResult {
        for soldier in self.battle_state.soldiers() {
            let color = if soldier.side() == self.gui_state.side() {
                GREEN
            } else {
                RED
            };

            let point = self
                .gui_state
                .window_point_from_world_point(soldier.world_point());
            mesh_builder.circle(DrawMode::fill(), point.to_vec2(), 2.0, 2.0, color)?;
        }

        Ok(())
    }

    /// Draw selection areas
    pub fn generate_areas_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        let cursor_world_point = self.gui_state.current_cursor_world_point();
        let cursor_window_point = self.gui_state.current_cursor_window_point();

        // Draw soldiers selection areas
        for soldier in self.battle_state.soldiers() {
            let rect = self
                .gui_state
                .window_rect_from_world_rect(self.graphics.soldier_selection_rect(soldier));
            mesh_builder.rectangle(DrawMode::stroke(1.0), rect, MAGENTA)?;
        }

        // Draw vehicle physics areas
        for vehicle in self.battle_state.vehicles() {
            let shape = self
                .gui_state
                .window_shape_from_world_shape(&vehicle.chassis_shape());

            mesh_builder.line(&shape.draw_points(), 1.0, MAGENTA)?;
        }

        // Draw selection area on cursor hover scene items
        for soldier_index in self.soldiers_at_point(cursor_world_point) {
            let soldier = self.battle_state.soldier(soldier_index);
            let rect = self
                .gui_state
                .window_rect_from_world_rect(self.graphics.soldier_selection_rect(soldier));
            mesh_builder.rectangle(DrawMode::stroke(1.0), rect, DARK_MAGENTA)?;
        }

        // Draw selection area on all order markers
        for (order, order_marker, _, world_point, _) in self.battle_state.order_markers(&Side::All)
        {
            let shape =
                self.gui_state
                    .window_shape_from_world_shape(&self.order_marker_selection_shape(
                        &order,
                        &order_marker,
                        &world_point,
                    ));
            let color = if shape.contains(cursor_window_point) {
                DARK_MAGENTA
            } else {
                MAGENTA
            };
            mesh_builder.line(&shape.draw_points(), 1.0, color)?;
        }

        Ok(())
    }

    ///
    pub fn generate_visibilities_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for squad_uuid in &self.gui_state.selected_squads().1 {
            let squad_composition = self.battle_state.squad(*squad_uuid);
            for soldier_index in squad_composition.members() {
                let from_soldier = self.battle_state.soldier(*soldier_index);
                let to_soldiers = self
                    .battle_state
                    .soldiers()
                    .iter()
                    .filter(|s| s.side() != from_soldier.side());
                for to_soldier in to_soldiers {
                    if let Some(visibility) = self
                        .battle_state
                        .visibilities()
                        .get(&(from_soldier.uuid(), to_soldier.uuid()))
                    {
                        let start_world_point = from_soldier.world_point();
                        let mut previous_point = self
                            .gui_state
                            .window_point_from_world_point(start_world_point);
                        let mut previous_opacity: f32 = 0.0;

                        for (segment_world_point, segment_new_opacity) in
                            visibility.opacity_segments.iter().skip(1)
                        {
                            let segment_point = self
                                .gui_state
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
    pub fn generate_targets_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for squad_uuid in &self.gui_state.selected_squads().1 {
            let squad_composition = self.battle_state.squad(*squad_uuid);
            for soldier_index in squad_composition.members() {
                let soldier = self.battle_state.soldier(*soldier_index);
                if let Some(target_soldier) = soldier.target() {
                    let target_soldier = self.battle_state.soldier(*target_soldier);
                    let from_point = self
                        .gui_state
                        .window_point_from_world_point(soldier.world_point());
                    let to_point = self
                        .gui_state
                        .window_point_from_world_point(target_soldier.world_point());
                    mesh_builder.line(&vec![from_point.to_vec2(), to_point.to_vec2()], 1.0, RED)?;
                }
            }
        }

        Ok(())
    }

    pub fn generate_debug_physics(&self, from: WorldPoint, to: WorldPoint) -> Vec<EngineMessage> {
        let mut messages = vec![];

        match self.gui_state.debug_physics() {
            DebugPhysics::None => {}
            DebugPhysics::MosinNagantM1924GunFire => {
                let weapon = Weapon::MosinNagantM1924(true, None);
                messages.extend(
                    [vec![EngineMessage::BattleState(
                        BattleStateMessage::PushBulletFire(BulletFire::new(
                            from,
                            to,
                            None,
                            weapon.ammunition(),
                            weapon.gun_fire_sound_type(),
                        )),
                    )]]
                    .concat(),
                );
            }
            DebugPhysics::BrandtMle2731Shelling => {
                messages.push(EngineMessage::BattleState(
                    BattleStateMessage::PushExplosion(Explosion::new(
                        from,
                        ExplosiveType::FA19241927,
                    )),
                ));
            }
        };

        messages
    }

    pub fn generate_physics_areas_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if let Some(explosive) = &self.gui_state.debug_physics().explosive() {
            let explosion = Explosion::new(
                self.gui_state.current_cursor_world_point(),
                explosive.clone(),
            );
            self.generate_explosive_areas_meshes(mesh_builder, &explosion)?;
        };

        for explosion in self.battle_state.explosions() {
            self.generate_explosive_areas_meshes(mesh_builder, explosion)?;
        }

        for bullet in self.battle_state.bullet_fires() {
            self.generate_bullet_areas_meshes(mesh_builder, bullet)?;
        }

        Ok(())
    }

    pub fn generate_explosive_areas_meshes(
        &self,
        mesh_builder: &mut MeshBuilder,
        explosion: &Explosion,
    ) -> GameResult {
        if let (
            Some(direct_death_rayons),
            Some(regressive_death_rayon),
            Some(regressive_injured_rayon),
        ) = (
            self.server_config
                .explosive_direct_death_rayon
                .get(explosion.type_()),
            self.server_config
                .explosive_regressive_death_rayon
                .get(explosion.type_()),
            self.server_config
                .explosive_regressive_injured_rayon
                .get(explosion.type_()),
        ) {
            let point = self
                .gui_state
                .window_point_from_world_point(*explosion.point());
            let direct_death_radius = self.gui_state.distance_pixels(&direct_death_rayons);
            mesh_builder.circle(
                DrawMode::stroke(1.0),
                point.to_vec2(),
                direct_death_radius * self.gui_state.zoom.factor(),
                1.0,
                RED,
            )?;

            let regressive_death_radius = self.gui_state.distance_pixels(regressive_death_rayon);
            let part = regressive_death_radius / 10.;
            for i in 1..=10 {
                let radius_ = part * i as f32;
                if radius_ > direct_death_radius {
                    mesh_builder.circle(
                        DrawMode::stroke(1.0),
                        point.to_vec2(),
                        radius_ * self.gui_state.zoom.factor(),
                        1.0,
                        Color {
                            r: 1.0,
                            g: 0.1,
                            b: 0.0,
                            a: 1.0 - (i as f32 / 15.),
                        },
                    )?;
                }
            }

            let regressive_injured_radius =
                self.gui_state.distance_pixels(regressive_injured_rayon);
            let part = regressive_injured_radius / 10.;
            for i in 1..=10 {
                let radius_ = part * i as f32;
                if radius_ > direct_death_radius {
                    mesh_builder.circle(
                        DrawMode::stroke(1.0),
                        point.to_vec2(),
                        radius_ * self.gui_state.zoom.factor(),
                        1.0,
                        Color {
                            r: 1.0,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0 - (i as f32 / 10.),
                        },
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn generate_bullet_areas_meshes(
        &self,
        mesh_builder: &mut MeshBuilder,
        bullet: &BulletFire,
    ) -> GameResult {
        let point = self
            .gui_state
            .window_point_from_world_point(*bullet.point());
        mesh_builder.circle(DrawMode::stroke(1.0), point.to_vec2(), 2., 1.0, RED)?;

        Ok(())
    }
}
