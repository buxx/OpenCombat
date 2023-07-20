use ggez::{
    graphics::{self, Canvas, DrawParam, MeshBuilder, Rect},
    Context, GameResult,
};

use battle_core::{
    entity::soldier::Soldier,
    game::squad::{squad_positions, Formation},
    order::{marker::OrderMarker, Order, PendingOrder},
    types::*,
};
use glam::Vec2;
use oc_core::spawn::SpawnZoneName;

use crate::{graphics::batch::QualifiedBatch, ui::hud::painter::HudPainter, utils::IntoSprite};

use super::{input::Control, Engine};

impl Engine {
    pub fn generate_soldiers_sprites(&mut self) -> GameResult {
        // All soldier ...
        for (i, soldier) in self.battle_state.soldiers().iter().enumerate() {
            // TODO : don't generate sprites of non visible soldiers (hidden enemy, outside screen, etc)
            if !self.is_soldier_drawable(SoldierIndex(i)) {
                continue;
            }

            let sprites = self
                .graphics
                .soldier_sprites(soldier, None, &self.gui_state.zoom);
            self.graphics
                .soldiers_mut()
                .extend(&self.gui_state.zoom, sprites);
        }

        // Dragged soldiers
        if let Some(squad_index) = &self.gui_state.dragged_squad() {
            let cursor = self.gui_state.current_cursor_world_point();
            let squad = self.battle_state.squad(*squad_index);
            let leader = self.battle_state.soldier(squad.leader());

            let sprites =
                self.graphics
                    .soldier_sprites(leader, Some(&cursor), &self.gui_state.zoom);
            self.graphics
                .soldiers_mut()
                .extend(&self.gui_state.zoom, sprites);

            let cursor_immobile_since =
                self.gui_state.frame_i() - self.gui_state.last_cursor_move_frame();
            if cursor_immobile_since >= 15 {
                for (member_id, formation_position) in
                    squad_positions(squad, Formation::Line, leader, Some(cursor))
                {
                    let soldier = self.battle_state.soldier(member_id);
                    let sprites = self.graphics.soldier_sprites(
                        soldier,
                        Some(&formation_position),
                        &self.gui_state.zoom,
                    );
                    self.graphics
                        .soldiers_mut()
                        .extend(&self.gui_state.zoom, sprites);
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
        if soldier.side() != self.gui_state.side()
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
            let sprites =
                self.graphics
                    .vehicle_sprites(VehicleIndex(i), vehicle, &self.gui_state.zoom);
            self.graphics
                .vehicles_mut()
                .extend(&self.gui_state.zoom, sprites);
        }

        Ok(())
    }

    pub fn generate_explosion_sprites(&mut self) -> GameResult {
        let sprites = self.graphics.explosion_sprites(&self.gui_state.zoom);
        self.graphics
            .explosions_mut()
            .extend(&self.gui_state.zoom, sprites);

        Ok(())
    }

    pub fn generate_map_sprites(&mut self, draw_decor: bool) -> GameResult {
        if self.battle_state.phase().placement() {
            self.generate_placement_map_sprites(draw_decor)?
        } else {
            self.generate_battle_map_sprites(draw_decor)?
        }
        Ok(())
    }

    pub fn generate_flags_sprites(&mut self) -> GameResult {
        for (flag_name, ownership) in self.battle_state.flags().ownerships() {
            let flag = self.battle_state.map().flag(flag_name);
            let mut draw_param = DrawParam::new()
                .src(Rect::from(ownership.to_relative_array()))
                .dest(flag.position().to_vec2() * self.gui_state.zoom.factor());
            if !self.gui_state.zoom.is_hd() {
                draw_param = draw_param.scale(Vec2::new(0.5, 0.5))
            }
            self.graphics.flags_mut().push(draw_param);
        }

        Ok(())
    }

    pub fn generate_placement_map_sprites(&mut self, _draw_decor: bool) -> GameResult {
        let (allowed_control, opponent_control) = self.zone_controls();
        let mut map_background_sprites = vec![];
        let mut dark_map_background_sprites = vec![];
        let mut map_dark_background_first = false;

        let all = DrawParam::new()
            .src(Rect::new(0.0, 0.0, 1.0, 1.0))
            .dest(Vec2::new(0., 0.) * self.gui_state.zoom.factor());
        if allowed_control.contains_spawn_zone(&SpawnZoneName::All) {
            map_background_sprites.push(all.clone());

            for opponent_spawn_zone in self
                .battle_state
                .map()
                .find_spawn_zones(opponent_control.spawn_zone_names())
            {
                dark_map_background_sprites.push(
                    DrawParam::new()
                        .src(Rect::new(
                            opponent_spawn_zone.relative_x(),
                            opponent_spawn_zone.relative_y(),
                            opponent_spawn_zone.relative_width(),
                            opponent_spawn_zone.relative_height(),
                        ))
                        .dest(
                            Vec2::new(opponent_spawn_zone.x(), opponent_spawn_zone.y())
                                * self.gui_state.zoom.factor(),
                        ),
                );
            }
        } else {
            dark_map_background_sprites.push(all.clone());
            map_dark_background_first = true;

            for allowed_spawn_zone in self
                .battle_state
                .map()
                .find_spawn_zones(allowed_control.spawn_zone_names())
            {
                if !opponent_control.contains_spawn_zone(allowed_spawn_zone.name()) {
                    map_background_sprites.push(
                        DrawParam::new()
                            .src(Rect::new(
                                allowed_spawn_zone.relative_x(),
                                allowed_spawn_zone.relative_y(),
                                allowed_spawn_zone.relative_width(),
                                allowed_spawn_zone.relative_height(),
                            ))
                            .dest(
                                Vec2::new(allowed_spawn_zone.x(), allowed_spawn_zone.y())
                                    * self.gui_state.zoom.factor(),
                            ),
                    );
                }
            }
        }

        for map_background_sprite in map_background_sprites {
            self.graphics
                .background_mut()
                .push(&self.gui_state.zoom, map_background_sprite);
        }

        for dark_map_background_sprite in dark_map_background_sprites {
            self.graphics
                .dark_background_mut()
                .push(&self.gui_state.zoom, dark_map_background_sprite);
        }
        self.graphics
            .set_map_dark_background_first(map_dark_background_first);

        Ok(())
    }

    pub fn generate_battle_map_sprites(&mut self, _draw_decor: bool) -> GameResult {
        self.graphics.background_mut().push(
            &self.gui_state.zoom,
            DrawParam::new()
                .src(Rect::new(0.0, 0.0, 1.0, 1.0))
                .dest(Vec2::new(0., 0.)),
        );
        Ok(())
    }

    pub fn generate_menu_sprites(&mut self) -> GameResult {
        if let Some((to_point, squad_ids)) = self.gui_state.squad_menu() {
            for sprite in self.graphics.squad_menu_sprites(
                *to_point,
                *self.gui_state.current_cursor_window_point(),
                squad_ids,
            ) {
                self.graphics.append_ui_batch(sprite);
            }
        }

        Ok(())
    }

    pub fn generate_hud_sprites(&mut self) -> GameResult {
        let sprites = HudPainter::new(&self.hud, &self.gui_state)
            .sprites()
            .clone();
        self.graphics.extend_ui_batch(sprites);

        Ok(())
    }

    pub fn generate_selection_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        self.generate_selected_entities_meshes(mesh_builder)?;

        Ok(())
    }

    pub fn generate_display_paths_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for display_paths in self.gui_state.display_paths() {
            for (display_path, _) in display_paths {
                self.generate_display_path_meshes(display_path, mesh_builder)?
            }
        }

        Ok(())
    }

    pub fn generate_game_play_meshes(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if self
            .gui_state
            .current_cursor_vector_window_points()
            .is_some()
            && self.gui_state.is_controlling(&Control::Soldiers)
        {
            self.generate_select_rectangle_meshes(mesh_builder)?;
        }

        Ok(())
    }

    pub fn generate_hud_meshes(
        &mut self,
        ctx: &Context,
        mesh_builder: &mut MeshBuilder,
    ) -> GameResult {
        HudPainter::new(&self.hud, &self.gui_state).meshes(ctx, mesh_builder)?;
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

        if self.gui_state.debug_targets {
            self.generate_targets_meshes(mesh_builder)?
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
                            .scale(self.gui_state.zoom.to_vec2()),
                    )
                }
                let cursor_point = self.gui_state.current_cursor_window_point();
                draw_params.push(
                    self.graphics
                        .order_marker_draw_params(&pending_order_marker, *cursor_point, Angle(0.))
                        .scale(self.gui_state.zoom.to_vec2()),
                );
            }
            PendingOrder::Defend(_) | PendingOrder::Hide(_) => {
                let pending_order_marker = self.pending_order_marker(pending_order);
                let to_point = self.gui_state.current_cursor_world_point().to_vec2();
                let from_point = squad_leader.world_point().to_vec2();
                let point = self
                    .gui_state
                    .window_point_from_world_point(squad_leader.world_point());
                draw_params.push(
                    self.graphics
                        .order_marker_draw_params(
                            &pending_order_marker,
                            point,
                            Angle::from_points(&to_point, &from_point),
                        )
                        // Defend/Hide sprite are scaled
                        .scale(self.gui_state.zoom.to_vec2()),
                )
            }
            PendingOrder::EngageOrFire(_) => {
                let pending_order_marker = self.pending_order_marker(pending_order);
                let to_point = self.gui_state.current_cursor_window_point();
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
                let cursor_point = self.gui_state.current_cursor_world_point();
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
            .scale(self.gui_state.zoom.to_vec2())]
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
            self.gui_state.debug_terrain(),
            draw_param,
        )?;

        Ok(())
    }
}
