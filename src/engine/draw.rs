use ggez::{
    graphics::{self, Canvas, DrawParam, MeshBuilder},
    Context, GameResult,
};

use crate::{
    debug::DebugTerrain,
    entity::soldier::Soldier,
    order::{marker::OrderMarker, Order, PendingOrder},
    types::*,
};

use super::{input::Control, Engine};

impl Engine {
    pub fn generate_soldiers_sprites(&mut self) -> GameResult {
        for (i, soldier) in self.shared_state.soldiers().iter().enumerate() {
            // TODO : don't generate sprites of non visible soldiers (hidden enemy, outside screen, etc)
            if !self.is_soldier_drawable(SoldierIndex(i)) {
                continue;
            }

            for sprite in self.graphics.soldier_sprites(SoldierIndex(i), soldier) {
                self.graphics.append_soldier_batch(sprite);
            }
        }

        Ok(())
    }

    fn is_soldier_drawable(&self, soldier_index: SoldierIndex) -> bool {
        let soldier = self.shared_state.soldier(soldier_index);

        // Don't draw soldier which inside vehicle
        if self.shared_state.soldier_board(soldier_index).is_some() {
            return false;
        }

        // Don't draw soldier in opposite side and not visible
        if soldier.get_side() != self.local_state.side()
            && !self.soldier_is_visible_by_side(soldier, self.local_state.side())
        {
            return false;
        }

        return true;
    }

    pub fn generate_vehicles_sprites(&mut self) -> GameResult {
        for (i, vehicle) in self.shared_state.vehicles().iter().enumerate() {
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
        if let Some((to_point, squad_id)) = self.local_state.get_squad_menu() {
            for sprite in self.graphics.squad_menu_sprites(
                *to_point,
                *self.local_state.get_current_cursor_window_point(),
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
        for (display_path, _) in self.local_state.get_display_paths() {
            self.generate_display_path_meshes(display_path, mesh_builder)?
        }

        Ok(())
    }

    pub fn generate_game_play_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if self.local_state.get_pending_order().is_none()
            && self.local_state.is_controlling(&Control::Soldiers)
        {
            self.generate_select_rectangle_meshes(mesh_builder)?;
        }

        Ok(())
    }

    pub fn generate_debug_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if self.local_state.debug_mouse {
            self.generate_debug_mouse_meshes(mesh_builder)?;
        }

        if self.local_state.debug_move_paths {
            self.generate_move_paths_meshes(mesh_builder)?
        }

        if self.local_state.debug_formation_positions {
            self.generate_formation_positions_meshes(mesh_builder)?
        }

        if self.local_state.debug_scene_item_circles {
            self.generate_scene_item_circles_meshes(mesh_builder)?
        }

        if self.local_state.debug_areas {
            self.generate_areas_meshes(mesh_builder)?
        }

        if self.local_state.debug_visibilities {
            self.generate_visibilities_meshes(mesh_builder)?
        }

        self.generate_debug_point_meshes(mesh_builder)?;

        Ok(())
    }

    pub fn generate_pending_order_sprites(&self, pending_order: &PendingOrder) -> Vec<DrawParam> {
        let mut draw_params = vec![];

        let squad = self.shared_state.squad(*pending_order.squad_index());
        let squad_leader = self.shared_state.soldier(squad.leader());

        match pending_order {
            PendingOrder::MoveTo(_, _, cached_points)
            | PendingOrder::MoveFastTo(_, _, cached_points)
            | PendingOrder::SneakTo(_, _, cached_points) => {
                let sprite_infos = self.pending_order_marker(pending_order).sprite_info();
                for cached_point in cached_points {
                    let point = self
                        .local_state
                        .window_point_from_world_point(*cached_point);
                    draw_params.push(
                        sprite_infos
                            .as_draw_params(point, Angle(0.), Offset::half())
                            .scale(self.local_state.display_scene_scale.to_vec2()),
                    )
                }
                let cursor_point = self.local_state.get_current_cursor_window_point();
                draw_params.push(
                    sprite_infos
                        .as_draw_params(*cursor_point, Angle(0.), Offset::half())
                        .scale(self.local_state.display_scene_scale.to_vec2()),
                );
            }
            PendingOrder::Defend(_) | PendingOrder::Hide(_) => {
                let sprite_infos = self.pending_order_marker(pending_order).sprite_info();
                let to_point = self.local_state.get_current_cursor_world_point().to_vec2();
                let from_point = squad_leader.get_world_point().to_vec2();
                let point = self
                    .local_state
                    .window_point_from_world_point(squad_leader.get_world_point());
                draw_params.push(
                    sprite_infos
                        .as_draw_params(
                            point,
                            Angle::from_points(&to_point, &from_point),
                            Offset::half(),
                        )
                        // Defend/Hide sprite are scaled
                        .scale(self.local_state.display_scene_scale.to_vec2()),
                )
            }
            PendingOrder::EngageOrFire(_) => {
                let sprite_infos = self.pending_order_marker(pending_order).sprite_info();
                let to_point = self.local_state.get_current_cursor_window_point();
                draw_params.push(sprite_infos.as_draw_params(*to_point, Angle(0.), Offset::half()))
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
                let cursor_point = self.local_state.get_current_cursor_world_point();
                if let Some(_) = self
                    .get_opponent_soldiers_at_point(cursor_point)
                    .iter()
                    .filter(|s| s.can_be_designed_as_target())
                    .filter(|s| self.soldier_is_visible_by_side(s, self.local_state.side()))
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
        let sprite_infos = order_marker.sprite_info();
        let angle = order.angle().unwrap_or(Angle(0.));
        vec![sprite_infos
            .as_draw_params(point, angle, Offset::half())
            // Defend/Hide sprite must be scaled
            .scale(self.local_state.display_scene_scale.to_vec2())]
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
            self.local_state.get_debug_terrain(),
            draw_param,
        )?;

        Ok(())
    }
}
