use ggez::{
    graphics::{self, Canvas, DrawParam, MeshBuilder},
    Context, GameResult,
};

use battle_core::{
    entity::soldier::Soldier,
    game::squad::{squad_positions, Formation},
    order::{marker::OrderMarker, Order, PendingOrder},
    types::*,
};

use super::{input::Control, Engine};

impl Engine {
    pub fn generate_soldiers_sprites(&mut self) -> GameResult {
        // All soldier ...
        for (i, soldier) in self.battle_state.soldiers().iter().enumerate() {
            // TODO : don't generate sprites of non visible soldiers (hidden enemy, outside screen, etc)
            if !self.is_soldier_drawable(SoldierIndex(i)) {
                continue;
            }

            for sprite in self.graphics.soldier_sprites(soldier, None) {
                self.graphics.append_soldier_batch(sprite);
            }
        }

        // Dragged soldiers
        if let Some(squad_index) = &self.gui_state.dragged_squad() {
            let cursor = self.gui_state.get_current_cursor_world_point();
            let squad = self.battle_state.squad(*squad_index);
            let leader = self.battle_state.soldier(squad.leader());
            for sprite in self.graphics.soldier_sprites(leader, Some(&cursor)) {
                self.graphics.append_soldier_batch(sprite);
            }

            let cursor_immobile_since =
                self.gui_state.get_frame_i() - self.gui_state.get_last_cursor_move_frame();
            if cursor_immobile_since >= 15 {
                for (member_id, formation_position) in
                    squad_positions(squad, Formation::Line, leader, Some(cursor))
                {
                    let soldier = self.battle_state.soldier(member_id);
                    for sprite in self
                        .graphics
                        .soldier_sprites(soldier, Some(&formation_position))
                    {
                        self.graphics.append_soldier_batch(sprite);
                    }
                }
            }
        }

        Ok(())
    }

    fn is_soldier_drawable(&self, soldier_index: SoldierIndex) -> bool {
        let soldier = self.battle_state.soldier(soldier_index);

        // Don't draw soldier which inside vehicle
        if self.battle_state.soldier_board(soldier_index).is_some() {
            return false;
        }

        // Don't draw soldier in opposite side and not visible
        if soldier.get_side() != self.gui_state.side()
            && !self
                .battle_state
                .soldier_is_visible_by_side(soldier, self.gui_state.side())
        {
            return false;
        }

        return true;
    }

    pub fn generate_vehicles_sprites(&mut self) -> GameResult {
        for (i, vehicle) in self.battle_state.vehicles().iter().enumerate() {
            for sprite in self.graphics.vehicle_sprites(VehicleIndex(i), vehicle) {
                self.graphics.append_vehicles_batch(sprite);
            }
        }

        Ok(())
    }

    pub fn generate_explosion_sprites(&mut self) -> GameResult {
        for sprite in self.graphics.explosion_sprites() {
            self.graphics.append_explosions_batch(sprite);
        }

        Ok(())
    }

    pub fn generate_map_sprites(&self, _draw_decor: bool) -> GameResult {
        // Note : Background sprites have been prepared once for map_background_batch
        // Note : Decor sprites have been prepared once for map_background_batch
        Ok(())
    }

    pub fn generate_menu_sprites(&mut self) -> GameResult {
        if let Some((to_point, squad_id)) = self.gui_state.get_squad_menu() {
            for sprite in self.graphics.squad_menu_sprites(
                *to_point,
                *self.gui_state.get_current_cursor_window_point(),
                *squad_id,
            ) {
                self.graphics.append_ui_batch(sprite);
            }
        }

        Ok(())
    }

    pub fn generate_selection_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        self.generate_selected_entities_meshes(mesh_builder)?;

        Ok(())
    }

    pub fn generate_display_paths_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for (display_path, _) in self.gui_state.get_display_paths() {
            self.generate_display_path_meshes(display_path, mesh_builder)?
        }

        Ok(())
    }

    pub fn generate_game_play_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if self.gui_state.get_pending_order().is_none()
            && self.gui_state.is_controlling(&Control::Soldiers)
        {
            self.generate_select_rectangle_meshes(mesh_builder)?;
        }

        Ok(())
    }

    pub fn generate_debug_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if self.gui_state.debug_mouse {
            self.generate_debug_mouse_meshes(mesh_builder)?;
        }

        if self.gui_state.debug_move_paths {
            self.generate_move_paths_meshes(mesh_builder)?
        }

        if self.gui_state.debug_formation_positions {
            self.generate_formation_positions_meshes(mesh_builder)?
        }

        if self.gui_state.debug_scene_item_circles {
            self.generate_scene_item_circles_meshes(mesh_builder)?
        }

        if self.gui_state.debug_areas {
            self.generate_areas_meshes(mesh_builder)?
        }

        if self.gui_state.debug_visibilities {
            self.generate_visibilities_meshes(mesh_builder)?
        }

        if self.gui_state.debug_physics_areas {
            self.generate_physics_areas_meshes(mesh_builder)?
        }

        self.generate_debug_point_meshes(mesh_builder)?;

        Ok(())
    }

    pub fn generate_pending_order_sprites(&self, pending_order: &PendingOrder) -> Vec<DrawParam> {
        let mut draw_params = vec![];

        let squad = self.battle_state.squad(*pending_order.squad_index());
        let squad_leader = self.battle_state.soldier(squad.leader());

        match pending_order {
            PendingOrder::MoveTo(_, _, cached_points)
            | PendingOrder::MoveFastTo(_, _, cached_points)
            | PendingOrder::SneakTo(_, _, cached_points) => {
                let pending_order_marker = self.pending_order_marker(pending_order);
                for cached_point in cached_points {
                    let point = self.gui_state.window_point_from_world_point(*cached_point);
                    draw_params.push(
                        self.graphics
                            .order_marker_draw_params(&pending_order_marker, point, Angle(0.))
                            .scale(self.gui_state.display_scene_scale.to_vec2()),
                    )
                }
                let cursor_point = self.gui_state.get_current_cursor_window_point();
                draw_params.push(
                    self.graphics
                        .order_marker_draw_params(&pending_order_marker, *cursor_point, Angle(0.))
                        .scale(self.gui_state.display_scene_scale.to_vec2()),
                );
            }
            PendingOrder::Defend(_) | PendingOrder::Hide(_) => {
                let pending_order_marker = self.pending_order_marker(pending_order);
                let to_point = self.gui_state.get_current_cursor_world_point().to_vec2();
                let from_point = squad_leader.get_world_point().to_vec2();
                let point = self
                    .gui_state
                    .window_point_from_world_point(squad_leader.get_world_point());
                draw_params.push(
                    self.graphics
                        .order_marker_draw_params(
                            &pending_order_marker,
                            point,
                            Angle::from_points(&to_point, &from_point),
                        )
                        // Defend/Hide sprite are scaled
                        .scale(self.gui_state.display_scene_scale.to_vec2()),
                )
            }
            PendingOrder::EngageOrFire(_) => {
                let pending_order_marker = self.pending_order_marker(pending_order);
                let to_point = self.gui_state.get_current_cursor_window_point();
                draw_params.push(self.graphics.order_marker_draw_params(
                    &pending_order_marker,
                    *to_point,
                    Angle(0.),
                ))
            }
        }

        draw_params
    }

    fn pending_order_marker(&self, pending_order: &PendingOrder) -> OrderMarker {
        match pending_order {
            PendingOrder::MoveTo(_, _, _) => OrderMarker::MoveTo,
            PendingOrder::MoveFastTo(_, _, _) => OrderMarker::MoveFastTo,
            PendingOrder::SneakTo(_, _, _) => OrderMarker::SneakTo,
            PendingOrder::Defend(_) => OrderMarker::Defend,
            PendingOrder::Hide(_) => OrderMarker::Hide,
            PendingOrder::EngageOrFire(_) => {
                let cursor_point = self.gui_state.get_current_cursor_world_point();
                if let Some(_) = self
                    .get_opponent_soldiers_at_point(cursor_point)
                    .iter()
                    .filter(|s| s.can_be_designed_as_target())
                    .filter(|s| {
                        self.battle_state
                            .soldier_is_visible_by_side(s, self.gui_state.side())
                    })
                    .collect::<Vec<&&Soldier>>()
                    .first()
                {
                    OrderMarker::EngageSquad
                } else {
                    OrderMarker::SuppressFire
                }
            }
        }
    }

    pub fn generate_order_marker_sprites(
        &self,
        order: &Order,
        order_marker: &OrderMarker,
        point: WindowPoint,
    ) -> Vec<DrawParam> {
        let angle = order.angle().unwrap_or(Angle(0.));
        vec![self
            .graphics
            .order_marker_draw_params(order_marker, point, angle)
            // Defend/Hide sprite must be scaled
            .scale(self.gui_state.display_scene_scale.to_vec2())]
    }

    pub fn draw_debug_terrain(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        draw_param: graphics::DrawParam,
    ) -> GameResult {
        self.graphics.draw_debug_terrain(
            ctx,
            canvas,
            self.gui_state.get_debug_terrain(),
            draw_param,
        )?;

        Ok(())
    }
}
