use ggez::{
    event::KeyCode,
    graphics::{Color, DrawMode, MeshBuilder, Rect, StrokeOptions},
    input, Context, GameResult,
};
use glam::Vec2;

use crate::{
    behavior::BehaviorMode,
    config::{
        DEFAULT_SELECTED_SQUARE_SIDE, DEFAULT_SELECTED_SQUARE_SIDE_HALF,
        PENDING_ORDER_PATH_FINDING_DRAW_FRAMES,
    },
    message::*,
    order::{Order, PendingOrder},
    types::*,
    ui::menu::squad_menu_sprite_info,
    utils::GREEN,
};

use super::{input::Control, Engine};

impl Engine {
    pub fn generate_selected_entities_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for squad_uuid in self.local_state.selected_squads() {
            for soldier_index in self.shared_state.squad(*squad_uuid).members() {
                let soldier = self.shared_state.soldier(*soldier_index);
                let point = self
                    .local_state
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
        if let Some((start, end)) = self.local_state.current_cursor_vector_window_points() {
            mesh_builder.rectangle(
                DrawMode::stroke(1.0),
                Rect::new(start.x, start.y, end.x - start.x, end.y - start.y),
                GREEN,
            )?;
        }

        Ok(())
    }

    pub fn get_side_entities_at_point(&self, point: WorldPoint) -> Vec<SoldierIndex> {
        let soldier_indexes = self.get_soldiers_at_point(point);
        self.filter_entities_by_side(soldier_indexes)
    }

    fn digest_scene_select_by_click(&self, point: WindowPoint) -> Vec<Message> {
        let world_point = self.local_state.world_point_from_window_point(point);
        let soldier_indexes = self.get_side_entities_at_point(world_point);
        if soldier_indexes.len() > 0 {
            let squad_ids = self.squad_ids_from_entities(vec![soldier_indexes[0]]);
            return vec![Message::LocalState(LocalStateMessage::SetSelectedSquads(
                squad_ids,
            ))];
        };

        vec![Message::LocalState(LocalStateMessage::SetSelectedSquads(
            vec![],
        ))]
    }

    fn digest_squad_menu_select_by_click(&self, cursor_point: WindowPoint) -> Vec<Message> {
        let (menu_point, squad_id) = self
            .local_state
            .get_squad_menu()
            .expect("This code should only called when squad menu");
        let squad_menu_sprite_info = squad_menu_sprite_info();
        if let Some(menu_item) = squad_menu_sprite_info.item_clicked(&menu_point, &cursor_point) {
            return vec![Message::LocalState(LocalStateMessage::SetPendingOrder(
                Some((menu_item.to_pending_order(), squad_id, None, vec![])),
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
                .local_state
                .window_point_from_world_point(last_world_point);
            points.push(last_window_point.to_vec2());
            for world_point in &path.points {
                last_world_point = world_point.clone();
                let window_point = self.local_state.window_point_from_world_point(*world_point);
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
        if let Some((pending_order, squad_id, _, cached_points)) =
            self.local_state.get_pending_order()
        {
            let sprites =
                self.generate_pending_order_sprites(pending_order, *squad_id, cached_points);
            self.graphics.extend_ui_batch(sprites);
        }

        for (order, order_marker, squad_id, point, order_marker_index) in
            self.shared_state.order_markers(self.local_state.side())
        {
            // Special case : If we are dragging this order_marker_index, don't draw it (because we only want draw the
            // dragged order marker index)
            if let Some((_, pending_squad_id, pending_order_marker_index, _)) =
                self.local_state.get_pending_order()
            {
                if let Some(pending_order_marker_index_) = pending_order_marker_index {
                    if *pending_squad_id == squad_id
                        && *pending_order_marker_index_ == order_marker_index
                    {
                        continue;
                    }
                }
            }
            let window_point = self.local_state.window_point_from_world_point(point);
            let sprites = self.generate_order_marker_sprites(&order, &order_marker, window_point);
            self.graphics.extend_ui_batch(sprites);
        }

        Ok(())
    }

    pub fn order_from_pending_order(
        &self,
        pending_order: &PendingOrder,
        squad_id: SquadUuid,
        order_marker_index: Option<OrderMarkerIndex>,
        cached_points: &Vec<WorldPoint>,
    ) -> Option<Order> {
        match pending_order {
            crate::order::PendingOrder::MoveTo => {
                //
                self.create_move_to_order(squad_id, order_marker_index, cached_points)
            }
            crate::order::PendingOrder::MoveFastTo => {
                //
                self.create_move_fast_to_order(squad_id, order_marker_index, cached_points)
            }
            crate::order::PendingOrder::SneakTo => {
                //
                self.create_sneak_to_order(squad_id, order_marker_index, cached_points)
            }
            crate::order::PendingOrder::Defend => {
                //
                self.create_defend_order(squad_id)
            }
            crate::order::PendingOrder::Hide => {
                //
                self.create_hide_order(squad_id)
            }
        }
    }

    pub fn ui_events(&mut self, ctx: &Context) -> Vec<Message> {
        let mut messages = vec![];

        while let Some(event) = self.local_state.pop_ui_event() {
            match event {
                UIEvent::FinishedCursorLeftClick(point) => {
                    let squad_menu_displayed = self.local_state.get_squad_menu().is_some();

                    // If this is a map selection click
                    if !squad_menu_displayed {
                        messages.extend(self.digest_scene_select_by_click(point));
                    }

                    // If this is a squad menu click
                    if squad_menu_displayed {
                        messages.extend(self.digest_squad_menu_select_by_click(point));
                        messages.push(Message::LocalState(LocalStateMessage::SetSquadMenu(None)));
                    }

                    // This is a pending order click
                    if let Some((pending_order, squad_id, _, cached_points)) =
                        self.local_state.get_pending_order()
                    {
                        let is_appending = input::keyboard::is_key_pressed(ctx, KeyCode::LShift)
                            || input::keyboard::is_key_pressed(ctx, KeyCode::RShift);

                        if is_appending {
                            messages.extend(vec![Message::LocalState(
                                LocalStateMessage::AddCachePointToPendingOrder(
                                    self.local_state.get_current_cursor_world_point(),
                                ),
                            )]);
                        } else {
                            // If order produced, push it on shared state
                            if let Some(order_) = self.order_from_pending_order(
                                pending_order,
                                *squad_id,
                                None,
                                cached_points,
                            ) {
                                messages.push(Message::SharedState(
                                    SharedStateMessage::PushCommandOrder(*squad_id, order_),
                                ))
                            }

                            // In all cases, remove pending order
                            messages.extend(vec![Message::LocalState(
                                LocalStateMessage::SetPendingOrder(None),
                            )]);
                        }
                    };

                    // In all cases, clean some things
                    messages.extend(vec![Message::LocalState(
                        LocalStateMessage::SetDisplayPaths(vec![]),
                    )]);
                }
                UIEvent::FinishedCursorVector(start, end) => {
                    if !self.local_state.controlling(&Control::Map) {
                        if let Some((pending_order, squad_id, order_marker_index, _)) =
                            self.local_state.get_pending_order()
                        {
                            if let Some(order_) = self.order_from_pending_order(
                                pending_order,
                                *squad_id,
                                *order_marker_index,
                                &vec![],
                            ) {
                                messages.push(Message::SharedState(
                                    SharedStateMessage::PushCommandOrder(*squad_id, order_),
                                ))
                            }
                            messages.extend(vec![
                                Message::LocalState(LocalStateMessage::SetPendingOrder(None)),
                                Message::LocalState(LocalStateMessage::SetDisplayPaths(vec![])),
                            ]);
                        } else {
                            let world_start = self.local_state.world_point_from_window_point(start);
                            let world_end = self.local_state.world_point_from_window_point(end);
                            let soldier_indexes = self.get_entities_in_area(world_start, world_end);
                            let soldier_indexes = self.filter_entities_by_side(soldier_indexes);
                            let squad_ids = self.squad_ids_from_entities(soldier_indexes);
                            messages.push(Message::LocalState(
                                LocalStateMessage::SetSelectedSquads(squad_ids),
                            ));
                        }
                    }
                }
                UIEvent::FinishedCursorRightClick(point) => {
                    let world_point = self.local_state.world_point_from_window_point(point);
                    let soldier_indexes = self.get_side_entities_at_point(world_point);
                    let mut squad_id: Option<SquadUuid> = None;

                    // If squad under cursor, select it
                    if soldier_indexes.len() > 0 {
                        let squad_id_ = self.squad_ids_from_entities(soldier_indexes.clone())[0];
                        squad_id = Some(squad_id_);
                        messages.push(Message::LocalState(LocalStateMessage::SetSelectedSquads(
                            vec![squad_id_],
                        )));

                    // Else, if squads already selected, keep only one
                    } else if self.local_state.selected_squads().len() > 0 {
                        squad_id = Some(self.local_state.selected_squads()[0]);
                        messages.push(Message::LocalState(LocalStateMessage::SetSelectedSquads(
                            vec![self.local_state.selected_squads()[0]],
                        )));
                    }

                    // Display a squad menu if squad under cursor or selected squad
                    if let Some(squad_id_) = squad_id {
                        messages.push(Message::LocalState(LocalStateMessage::SetSquadMenu(Some(
                            (
                                self.local_state.get_current_cursor_window_point().clone(),
                                squad_id_,
                            ),
                        ))));
                    }
                }
                UIEvent::ImmobileCursorSince(since) => {
                    // Paths to draw if pending order
                    if let Some((pending_order, squad_id, order_marker_index, cached_points)) =
                        self.local_state.get_pending_order()
                    {
                        if since == PENDING_ORDER_PATH_FINDING_DRAW_FRAMES {
                            if pending_order.expect_path_finding() {
                                messages.push(Message::LocalState(LocalStateMessage::PushUIEvent(
                                    UIEvent::DrawPathFinding(
                                        *squad_id,
                                        *order_marker_index,
                                        cached_points.clone(),
                                    ),
                                )));
                            }
                        }
                    }
                }
                UIEvent::DrawPathFinding(squad_id, order_marker_index, cached_points) => {
                    let squad_leader_index = self.shared_state.squad(squad_id).leader();
                    let behavior_mode = self.soldier_behavior_mode(squad_leader_index);
                    let take_only_bounds = behavior_mode == BehaviorMode::Vehicle;

                    if let Some(world_paths) = self.create_path_finding(
                        squad_id,
                        order_marker_index,
                        &cached_points,
                        take_only_bounds,
                    ) {
                        messages.push(Message::LocalState(LocalStateMessage::SetDisplayPaths(
                            vec![(world_paths, squad_id)],
                        )));
                    }
                }
                UIEvent::CursorMove(_point) => {
                    messages.push(Message::LocalState(LocalStateMessage::SetDisplayPaths(
                        vec![],
                    )));
                }
            }
        }

        messages
    }
}
