use ggez::{
    graphics::{self, DrawParam, MeshBuilder},
    Context, GameResult,
};

use crate::{
    debug::DebugTerrain,
    order::{marker::OrderMarker, Order, PendingOrder},
    types::*,
};

use super::Engine;

impl Engine {
    // TODO : don't generate sprites of non visible soldiers (hidden enemy, outside screen, etc)
    pub fn generate_soldiers_sprites(&mut self) -> GameResult {
        for (i, soldier) in self.shared_state.soldiers().iter().enumerate() {
            if !self.is_soldier_drawable(SoldierIndex(i)) {
                continue;
            }

            for sprite in self.graphics.soldier_sprites(SoldierIndex(i), soldier) {
                self.graphics.append_sprites_batch(sprite);
            }
        }

        Ok(())
    }

    fn is_soldier_drawable(&self, soldier_index: SoldierIndex) -> bool {
        if self.shared_state.soldier_board(soldier_index).is_some() {
            return false;
        }

        return true;
    }

    pub fn generate_vehicles_sprites(&mut self) -> GameResult {
        for (i, vehicle) in self.shared_state.vehicles().iter().enumerate() {
            for sprite in self.graphics.vehicle_sprites(VehicleIndex(i), vehicle) {
                self.graphics.append_sprites_batch(sprite);
            }
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
        if self.local_state.get_pending_order().is_none() {
            self.generate_select_rectangle_meshes(mesh_builder)?;
        }

        Ok(())
    }

    pub fn generate_debug_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if self.local_state.get_debug_level().mouse() {
            self.generate_debug_mouse_meshes(mesh_builder)?;
        }

        if self.local_state.get_debug_level().move_paths() {
            self.generate_move_paths_meshes(mesh_builder)?
        }

        if self.local_state.get_debug_level().formation_positions() {
            self.generate_formation_positions_meshes(mesh_builder)?
        }

        if self.local_state.get_debug_level().scene_item_circles() {
            self.generate_scene_item_circles_meshes(mesh_builder)?
        }

        if self.local_state.get_debug_level().areas() {
            self.generate_areas_meshes(mesh_builder)?
        }

        self.generate_debug_point_meshes(mesh_builder)?;

        Ok(())
    }

    pub fn generate_pending_order_sprites(
        &self,
        pending_order: &PendingOrder,
        squad_id: SquadUuid,
        cached_points: &Vec<WorldPoint>,
    ) -> Vec<DrawParam> {
        let mut draw_params = vec![];
        let order_marker = pending_order.marker();
        let sprite_infos = order_marker.sprite_info();
        for (draw_to, angle, offset) in
            self.get_pending_order_params(pending_order, squad_id, cached_points)
        {
            draw_params.push(sprite_infos.as_draw_params(draw_to, angle, offset))
        }
        draw_params
    }

    pub fn generate_order_marker_sprites(
        &self,
        order: &Order,
        order_marker: &OrderMarker,
        point: WindowPoint,
    ) -> Vec<DrawParam> {
        let sprite_infos = order_marker.sprite_info();
        let offset = order_marker.offset();
        let angle = order.angle().unwrap_or(Angle(0.));
        vec![sprite_infos.as_draw_params(point, angle, offset)]
    }

    pub fn draw_debug_terrain(
        &self,
        ctx: &mut Context,
        draw_param: graphics::DrawParam,
    ) -> GameResult {
        match self.local_state.get_debug_terrain() {
            DebugTerrain::Tiles => {
                if let Some(debug_terrain_batch) = &self.map.debug_terrain_batch {
                    graphics::draw(ctx, debug_terrain_batch, draw_param)?;
                }
            }
            DebugTerrain::Opacity => {
                if let Some(debug_terrain_opacity_mesh_builder) =
                    &self.map.debug_terrain_opacity_mesh_builder
                {
                    let debug_terrain_opacity_mesh =
                        debug_terrain_opacity_mesh_builder.build(ctx)?;
                    graphics::draw(ctx, &debug_terrain_opacity_mesh, draw_param)?;
                }
            }
            DebugTerrain::None => {}
        };

        Ok(())
    }
}
