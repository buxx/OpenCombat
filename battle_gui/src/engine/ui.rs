use ggez::{
    graphics::{Color, DrawMode, MeshBuilder, Rect, StrokeOptions},
    winit::event::VirtualKeyCode,
    Context, GameResult,
};
use glam::Vec2;

use battle_core::{
    audio::Sound,
    config::{
        DEFAULT_SELECTED_SQUARE_SIDE, DEFAULT_SELECTED_SQUARE_SIDE_HALF,
        PENDING_ORDER_PATH_FINDING_DRAW_FRAMES,
    },
    entity::soldier::Soldier,
    game::cover::CoverFinder,
    order::{Order, PendingOrder},
    state::battle::message::{BattleStateMessage, SoldierMessage, VehicleMessage},
    types::*,
    utils::DebugPoint,
};

use crate::{engine::event::UIEvent, ui::menu::squad_menu_sprite_info, utils::GREEN};

use super::{
    input::Control,
    message::{EngineMessage, GuiStateMessage},
    Engine,
};

impl Engine {
    pub fn generate_selected_entities_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for squad_uuid in &self.gui_state.selected_squads().1 {
            for soldier_index in self.battle_state.squad(*squad_uuid).members() {
                let soldier = self.battle_state.soldier(*soldier_index);
                let point = self
                    .gui_state
                    .window_point_from_world_point(soldier.get_world_point());
                let rect = Rect::new(
                    point.x - DEFAULT_SELECTED_SQUARE_SIDE_HALF,
                    point.y - DEFAULT_SELECTED_SQUARE_SIDE_HALF,
                    DEFAULT_SELECTED_SQUARE_SIDE,
                    DEFAULT_SELECTED_SQUARE_SIDE,
                );
                mesh_builder.rectangle(DrawMode::Stroke(StrokeOptions::default()), rect, GREEN)?;
            }
        }

        Ok(())
    }

    pub fn generate_select_rectangle_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if self.gui_state.dragged_squad().is_some() {
            return Ok(());
        }

        if let Some((start, end)) = self.gui_state.current_cursor_vector_window_points() {
            mesh_builder.rectangle(
                DrawMode::stroke(1.0),
                Rect::new(start.x, start.y, end.x - start.x, end.y - start.y),
                GREEN,
            )?;
        }

        Ok(())
    }

    pub fn get_opponent_soldiers_at_point(&self, point: WorldPoint) -> Vec<&Soldier> {
        let soldier_indexes = self.get_soldiers_at_point(point);
        self.filter_entities_by_side(soldier_indexes, self.gui_state.opponent_side())
            .iter()
            .map(|i| self.battle_state.soldier(*i))
            .collect()
    }

    fn digest_scene_select_by_click(&self, point: WindowPoint) -> Vec<EngineMessage> {
        let world_point = self.gui_state.world_point_from_window_point(point);
        let soldier_indexes = self.get_soldiers_at_point(world_point);
        if soldier_indexes.len() > 0 {
            let squad_ids = self.squad_ids_from_entities(vec![soldier_indexes[0]]);
            return vec![EngineMessage::GuiState(GuiStateMessage::SetSelectedSquads(
                Some(soldier_indexes[0]),
                squad_ids,
            ))];
        };

        vec![EngineMessage::GuiState(GuiStateMessage::SetSelectedSquads(
            None,
            vec![],
        ))]
    }

    fn digest_squad_menu_select_by_click(
        &self,
        cursor_point: &WindowPoint,
        squad_menu_point: &WindowPoint,
        squad_index: &SquadUuid,
    ) -> Vec<EngineMessage> {
        let squad_menu_sprite_info = squad_menu_sprite_info();
        if let Some(menu_item) =
            squad_menu_sprite_info.item_clicked(&squad_menu_point, &cursor_point)
        {
            return vec![EngineMessage::GuiState(GuiStateMessage::SetPendingOrder(
                Some(menu_item.to_pending_order(squad_index)),
            ))];
        };

        vec![]
    }

    pub fn generate_display_path_meshes(
        &self,
        display_path: &WorldPaths,
        mesh_builder: &mut MeshBuilder,
    ) -> GameResult {
        let mut points: Vec<Vec2> = vec![];
        let mut last_world_point = display_path
            .next_point()
            .expect("Path must contains at least one point");
        for path in &display_path.paths {
            let last_window_point = self
                .gui_state
                .window_point_from_world_point(last_world_point);
            points.push(last_window_point.to_vec2());
            for world_point in &path.points {
                last_world_point = world_point.clone();
                let window_point = self.gui_state.window_point_from_world_point(*world_point);
                points.push(window_point.to_vec2())
            }
        }

        mesh_builder.line(
            &points,
            2.0,
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.2,
            },
        )?;

        Ok(())
    }

    pub fn generate_orders_sprites(&mut self) -> GameResult {
        if let Some(pending_order) = self.gui_state.get_pending_order() {
            let sprites = self.generate_pending_order_sprites(pending_order);
            self.graphics.extend_ui_batch(sprites);
        }

        for (order, order_marker, squad_id, point, order_marker_index) in
            self.battle_state.order_markers(self.gui_state.side())
        {
            // Special case : If we are dragging this order_marker_index, don't draw it (because we only want draw the
            // dragged order marker index)
            if let Some(pending_order) = self.gui_state.get_pending_order() {
                if let Some(pending_order_marker_index_) = pending_order.order_marker_index() {
                    if *pending_order.squad_index() == squad_id
                        && &order_marker_index == pending_order_marker_index_
                        && *pending_order_marker_index_ == order_marker_index
                    {
                        continue;
                    }
                }
            }
            let window_point = self.gui_state.window_point_from_world_point(point);
            let sprites = self.generate_order_marker_sprites(&order, &order_marker, window_point);
            self.graphics.extend_ui_batch(sprites);
        }

        Ok(())
    }

    pub fn order_from_pending_order(&self, pending_order: &PendingOrder) -> Option<Order> {
        match pending_order {
            PendingOrder::MoveTo(squad_index, order_marker_index, cached_points) => {
                //
                self.create_move_to_order(squad_index, order_marker_index, cached_points)
            }
            PendingOrder::MoveFastTo(squad_index, order_marker_index, cached_points) => {
                //
                self.create_move_fast_to_order(squad_index, order_marker_index, cached_points)
            }
            PendingOrder::SneakTo(squad_index, order_marker_index, cached_points) => {
                //
                self.create_sneak_to_order(squad_index, order_marker_index, cached_points)
            }
            PendingOrder::Defend(squad_index) => {
                //
                self.create_defend_order(*squad_index)
            }
            PendingOrder::Hide(squad_index) => {
                //
                self.create_hide_order(*squad_index)
            }
            PendingOrder::EngageOrFire(squad_index) => {
                //
                self.create_engage_order(&squad_index)
            }
        }
    }

    pub fn ui_events(&mut self, ctx: &Context) -> Vec<EngineMessage> {
        puffin::profile_scope!("ui_events");
        let mut messages = vec![];

        while let Some(event) = self.gui_state.pop_ui_event() {
            match event {
                UIEvent::FinishedCursorLeftClick(point) => {
                    match self.gui_state.controlling() {
                        Control::Soldiers => messages
                            .extend(self.left_click_finished_controlling_soldier(ctx, point)),
                        Control::Map => {}
                        Control::Physics => messages
                            .extend(self.left_click_finished_controlling_physics(ctx, point)),
                    };
                }
                UIEvent::FinishedCursorVector(start, end) => {
                    match self.gui_state.controlling() {
                        Control::Soldiers => messages.extend(
                            self.cursor_vector_finished_controlling_soldier(ctx, start, end),
                        ),
                        Control::Map => {}
                        Control::Physics => messages.extend(
                            self.cursor_vector_finished_controlling_physics(ctx, start, end),
                        ),
                    };
                }
                UIEvent::FinishedCursorRightClick(point) => {
                    let world_point = self.gui_state.world_point_from_window_point(point);
                    let soldier_indexes = self.get_soldiers_at_point(world_point);
                    let mut squad_id: Option<SquadUuid> = None;

                    // If squad under cursor, select it
                    if soldier_indexes.len() > 0 {
                        let squad_id_ = self.squad_ids_from_entities(soldier_indexes.clone())[0];
                        squad_id = Some(squad_id_);
                        messages.push(EngineMessage::GuiState(GuiStateMessage::SetSelectedSquads(
                            None,
                            vec![squad_id_],
                        )));

                    // Else, if squads already selected, keep only one
                    } else if self.gui_state.selected_squads().1.len() > 0 {
                        squad_id = Some(self.gui_state.selected_squads().1[0]);
                        messages.push(EngineMessage::GuiState(GuiStateMessage::SetSelectedSquads(
                            None,
                            vec![self.gui_state.selected_squads().1[0]],
                        )));
                    }

                    // Display a squad menu if squad under cursor or selected squad
                    if let Some(squad_id_) = squad_id {
                        messages.push(EngineMessage::GuiState(GuiStateMessage::SetSquadMenu(
                            Some((
                                self.gui_state.get_current_cursor_window_point().clone(),
                                squad_id_,
                            )),
                        )));
                    }
                }
                UIEvent::ImmobileCursorSince(since) => {
                    // Paths to draw if pending order
                    if let Some(pending_order) = self.gui_state.get_pending_order() {
                        if since == PENDING_ORDER_PATH_FINDING_DRAW_FRAMES {
                            if pending_order.expect_path_finding() {
                                messages.push(EngineMessage::GuiState(
                                    GuiStateMessage::PushUIEvent(UIEvent::DrawPathFinding(
                                        *pending_order.squad_index(),
                                        *pending_order.order_marker_index(),
                                        pending_order.cached_points().clone(),
                                    )),
                                ));
                            }
                        }
                    }
                }
                UIEvent::DrawPathFinding(squad_id, order_marker_index, cached_points) => {
                    let (path_mode, start_direction) =
                        self.battle_state.squad_path_mode_and_direction(squad_id);

                    if let Some(world_paths) = self.create_path_finding(
                        squad_id,
                        &order_marker_index,
                        &cached_points,
                        &path_mode,
                        &start_direction,
                    ) {
                        messages.push(EngineMessage::GuiState(GuiStateMessage::SetDisplayPaths(
                            vec![(world_paths, squad_id)],
                        )));
                    }
                }
                UIEvent::CursorMove(_point) => {
                    messages.push(EngineMessage::GuiState(GuiStateMessage::SetDisplayPaths(
                        vec![],
                    )));
                }
                UIEvent::DropSquad(squad_index, world_point) => {
                    messages.extend(self.drop_squad_to(&squad_index, &world_point))
                }
            }
        }

        messages
    }

    fn left_click_finished_controlling_soldier(
        &mut self,
        ctx: &Context,
        point: WindowPoint,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        if let Some((squad_menu_point, squad_index)) = self.gui_state.get_squad_menu() {
            messages.extend(self.digest_squad_menu_select_by_click(
                &point,
                squad_menu_point,
                squad_index,
            ));
            messages.push(EngineMessage::GuiState(GuiStateMessage::SetSquadMenu(None)));
        } else {
            messages.extend(self.digest_scene_select_by_click(point));
        }

        // This is a pending order click
        if let Some(pending_order) = self.gui_state.get_pending_order() {
            let is_appending = ctx.keyboard.is_key_pressed(VirtualKeyCode::LShift)
                || ctx.keyboard.is_key_pressed(VirtualKeyCode::RShift);

            if is_appending {
                messages.extend(vec![EngineMessage::GuiState(
                    GuiStateMessage::AddCachePointToPendingOrder(
                        self.gui_state.get_current_cursor_world_point(),
                    ),
                )]);
            } else {
                // If order produced, push it on shared state
                if let Some(order) = self.order_from_pending_order(pending_order) {
                    let squad_leader = self
                        .battle_state
                        .squad(*pending_order.squad_index())
                        .leader();
                    messages.extend(vec![
                        EngineMessage::BattleState(BattleStateMessage::Soldier(
                            squad_leader,
                            SoldierMessage::SetOrder(order),
                        )),
                        EngineMessage::PlaySound(Sound::Clac1),
                    ])
                } else {
                    messages.push(EngineMessage::PlaySound(Sound::Bip1))
                }

                // In all cases, remove pending order
                messages.extend(vec![EngineMessage::GuiState(
                    GuiStateMessage::SetPendingOrder(None),
                )]);
            }
        };

        // In all cases, clean some things
        messages.extend(vec![EngineMessage::GuiState(
            GuiStateMessage::SetDisplayPaths(vec![]),
        )]);

        messages
    }

    fn left_click_finished_controlling_physics(
        &mut self,
        _ctx: &Context,
        point: WindowPoint,
    ) -> Vec<EngineMessage> {
        let world_point = self.gui_state.world_point_from_window_point(point);
        self.generate_debug_physics(world_point.clone(), world_point)
    }

    fn cursor_vector_finished_controlling_soldier(
        &mut self,
        _ctx: &Context,
        start: WindowPoint,
        end: WindowPoint,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        if let Some(pending_order) = self.gui_state.get_pending_order() {
            if let Some(order_) = self.order_from_pending_order(pending_order) {
                let squad_leader = self
                    .battle_state
                    .squad(*pending_order.squad_index())
                    .leader();
                messages.extend(vec![
                    EngineMessage::BattleState(BattleStateMessage::Soldier(
                        squad_leader,
                        SoldierMessage::SetOrder(order_),
                    )),
                    EngineMessage::PlaySound(Sound::Clac1),
                ])
            } else {
                messages.push(EngineMessage::PlaySound(Sound::Bip1))
            }
            messages.extend(vec![
                EngineMessage::GuiState(GuiStateMessage::SetPendingOrder(None)),
                EngineMessage::GuiState(GuiStateMessage::SetDisplayPaths(vec![])),
            ]);
        } else {
            let world_start = self.gui_state.world_point_from_window_point(start);
            let world_end = self.gui_state.world_point_from_window_point(end);
            let soldier_indexes = self.get_entities_in_area(world_start, world_end);
            let soldier_indexes =
                self.filter_entities_by_side(soldier_indexes, self.gui_state.side());
            let squad_ids = self.squad_ids_from_entities(soldier_indexes);
            messages.push(EngineMessage::GuiState(GuiStateMessage::SetSelectedSquads(
                None, squad_ids,
            )));
        }

        messages
    }

    fn cursor_vector_finished_controlling_physics(
        &mut self,
        _ctx: &Context,
        start: WindowPoint,
        end: WindowPoint,
    ) -> Vec<EngineMessage> {
        let world_start = self.gui_state.world_point_from_window_point(start);
        let world_end = self.gui_state.world_point_from_window_point(end);
        self.generate_debug_physics(world_start, world_end)
    }

    fn drop_squad_to(&self, squad_index: &SquadUuid, point: &WorldPoint) -> Vec<EngineMessage> {
        let squad = self.battle_state.squad(*squad_index);
        if let Some(vehicle_index) = self.battle_state.soldier_vehicle(squad.leader()) {
            self.drop_vehicle_to(&vehicle_index, point)
        } else {
            self.drop_pedestrian_squad_to(squad_index, point)
        }
    }

    fn drop_vehicle_to(
        &self,
        vehicle_index: &VehicleIndex,
        point: &WorldPoint,
    ) -> Vec<EngineMessage> {
        vec![EngineMessage::BattleState(BattleStateMessage::Vehicle(
            *vehicle_index,
            VehicleMessage::SetWorldPosition(point.clone()),
        ))]
    }

    fn drop_pedestrian_squad_to(
        &self,
        squad_index: &SquadUuid,
        point: &WorldPoint,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];
        let squad = self.battle_state.squad(*squad_index);
        let leader = self.battle_state.soldier(squad.leader());
        let cursor_grid_point = self.battle_state.map().grid_point_from_world_point(&point);
        let (moves, debug_points) = CoverFinder::new(&self.battle_state, &self.server_config)
            .point(Some(point.clone()))
            .exclude_grid_points(vec![cursor_grid_point])
            .find_cover_points(squad, leader);

        messages.push(EngineMessage::BattleState(BattleStateMessage::Soldier(
            squad.leader(),
            SoldierMessage::SetWorldPosition(*point),
        )));
        for (member_id, _, cover_world_point) in moves {
            messages.push(EngineMessage::BattleState(BattleStateMessage::Soldier(
                member_id,
                SoldierMessage::SetWorldPosition(cover_world_point),
            )))
        }

        for new_debug_point in debug_points {
            messages.push(EngineMessage::GuiState(GuiStateMessage::PushDebugPoint(
                DebugPoint {
                    frame_i: self.gui_state.get_frame_i() + 120,
                    point: new_debug_point.point,
                },
            )))
        }

        messages
    }
}
