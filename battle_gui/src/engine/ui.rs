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
    game::{cover::CoverFinder, health::SoldierHealthBuilder},
    graphics::vehicle::VehicleGraphicInfos,
    order::{Order, PendingOrder},
    physics::path::Direction,
    state::battle::message::{BattleStateMessage, SoldierMessage, VehicleMessage},
    types::*,
    utils::DebugPoint,
};

use crate::{
    engine::event::UIEvent,
    ui::{color::Colorized, hud::HUD_HEIGHT, menu::squad_menu_sprite_info, BORDER_SIZE},
    utils::GREEN,
};

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
                    .window_point_from_world_point(soldier.world_point());

                mesh_builder.rectangle(
                    DrawMode::Stroke(StrokeOptions::default()),
                    Rect::new(
                        point.x - DEFAULT_SELECTED_SQUARE_SIDE_HALF,
                        point.y - DEFAULT_SELECTED_SQUARE_SIDE_HALF,
                        DEFAULT_SELECTED_SQUARE_SIDE,
                        DEFAULT_SELECTED_SQUARE_SIDE,
                    ),
                    SoldierHealthBuilder::new(soldier).build().color(),
                )?;
                if self.battle_state.squad(soldier.squad_uuid()).leader() == soldier.uuid() {
                    mesh_builder.rectangle(
                        DrawMode::Stroke(StrokeOptions::default()),
                        Rect::new(
                            point.x - DEFAULT_SELECTED_SQUARE_SIDE_HALF - 1.,
                            point.y - DEFAULT_SELECTED_SQUARE_SIDE_HALF - 1.,
                            DEFAULT_SELECTED_SQUARE_SIDE + 2.,
                            DEFAULT_SELECTED_SQUARE_SIDE + 2.,
                        ),
                        Color::BLUE,
                    )?;
                }
                if self.gui_state.selected_squads().0 == Some(*soldier_index) {
                    mesh_builder.rectangle(
                        DrawMode::Stroke(StrokeOptions::default()),
                        Rect::new(
                            point.x - DEFAULT_SELECTED_SQUARE_SIDE_HALF + 1.,
                            point.y - DEFAULT_SELECTED_SQUARE_SIDE_HALF + 1.,
                            DEFAULT_SELECTED_SQUARE_SIDE - 2.,
                            DEFAULT_SELECTED_SQUARE_SIDE - 2.,
                        ),
                        Color::WHITE,
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn generate_select_rectangle_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if self.gui_state.dragged_squad().is_some() {
            return Ok(());
        }

        if self.gui_state.dragged_squad().is_none() && self.gui_state.pending_order().is_empty() {
            if let Some((start, end)) = self.gui_state.current_cursor_vector_window_points() {
                mesh_builder.rectangle(
                    DrawMode::stroke(1.0),
                    Rect::new(start.x, start.y, end.x - start.x, end.y - start.y),
                    GREEN,
                )?;
            }
        }

        Ok(())
    }

    pub fn get_opponent_soldiers_at_point(&self, point: WorldPoint) -> Vec<&Soldier> {
        let soldiers = self.soldiers_at_point(point, Some(&self.gui_state.side().opposite()));
        let soldiers_indexes = soldiers.iter().map(|soldier| soldier.uuid()).collect();
        self.filter_entities_by_side(soldiers_indexes, self.gui_state.opponent_side())
            .iter()
            .map(|i| self.battle_state.soldier(*i))
            .collect()
    }

    fn digest_scene_select_by_click(
        &self,
        ctx: &Context,
        point: WindowPoint,
    ) -> Vec<EngineMessage> {
        let world_point = self.gui_state.world_point_from_window_point(point);
        let soldiers = self.soldiers_at_point(world_point, Some(self.gui_state.side()));
        if let Some(soldier) = soldiers.first() {
            let squad_ids = self.squad_ids_from_entities(vec![soldier.uuid()]);
            return vec![EngineMessage::GuiState(GuiStateMessage::SetSelectedSquads(
                Some(soldier.uuid()),
                squad_ids,
            ))];
        }

        // Little dirty but ensure it was not a click on the hud
        if !self.hud.contains(ctx, &[&point]) {
            return vec![EngineMessage::GuiState(GuiStateMessage::SetSelectedSquads(
                None,
                vec![],
            ))];
        }

        vec![]
    }

    fn digest_squad_menu_select_by_click(
        &self,
        cursor_point: &WindowPoint,
        squad_menu_point: &WindowPoint,
        squads: &[SquadUuid],
    ) -> Vec<EngineMessage> {
        let squad_menu_sprite_info = squad_menu_sprite_info();
        if let Some(menu_item) = squad_menu_sprite_info.item_clicked(squad_menu_point, cursor_point)
        {
            return vec![EngineMessage::GuiState(GuiStateMessage::SetPendingOrders(
                squads
                    .iter()
                    .map(|squad_index| menu_item.to_pending_order(squad_index))
                    .collect(),
            ))];
        };

        vec![]
    }

    fn digest_squad_menu_select_by_vector(
        &self,
        start_cursor_point: &WindowPoint,
        end_cursor_point: &WindowPoint,
        squad_menu_point: &WindowPoint,
        squads: &[SquadUuid],
    ) -> Option<Vec<EngineMessage>> {
        let squad_menu_sprite_info = squad_menu_sprite_info();
        if let (Some(menu_item), Some(_)) = (
            squad_menu_sprite_info.item_clicked(squad_menu_point, start_cursor_point),
            squad_menu_sprite_info.item_clicked(squad_menu_point, end_cursor_point),
        ) {
            return Some(vec![EngineMessage::GuiState(
                GuiStateMessage::SetPendingOrders(
                    squads
                        .iter()
                        .map(|squad_index| menu_item.to_pending_order(squad_index))
                        .collect(),
                ),
            )]);
        };

        None
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
                last_world_point = *world_point;
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

    pub fn generate_orders_sprites(&mut self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for pending_order in self.gui_state.pending_order() {
            let sprites = self.generate_pending_order_sprites(pending_order, mesh_builder)?;
            self.graphics.extend_ui_batch(sprites);
        }

        for (order, order_marker, _squad_id, point, _order_marker_index) in
            self.battle_state.order_markers(self.gui_state.side())
        {
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
                self.create_engage_order(squad_index)
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
                    let soldiers = self.soldiers_at_point(world_point, Some(self.gui_state.side()));
                    let mut squad_ids: Vec<SquadUuid> = vec![];

                    // If squad under cursor, select it
                    if !soldiers.is_empty() {
                        let soldiers_indexes =
                            soldiers.iter().map(|soldier| soldier.uuid()).collect();
                        let squad_id_ = self.squad_ids_from_entities(soldiers_indexes)[0];
                        squad_ids = vec![squad_id_];
                        messages.push(EngineMessage::GuiState(GuiStateMessage::SetSelectedSquads(
                            None,
                            vec![squad_id_],
                        )));

                    // Else, if squads already selected, keep only one
                    } else if !self.gui_state.selected_squads().1.is_empty() {
                        squad_ids = self.gui_state.selected_squads().1.clone();
                    }

                    if !self.gui_state.pending_order().is_empty() {
                        messages.extend([
                            EngineMessage::GuiState(GuiStateMessage::SetPendingOrders(vec![])),
                            EngineMessage::GuiState(GuiStateMessage::SetDisplayPaths(vec![])),
                            EngineMessage::PlaySound(Sound::Bip1),
                        ])
                        // Display a squad menu if squad under cursor or selected squad
                    } else if !squad_ids.is_empty() {
                        messages.push(EngineMessage::GuiState(GuiStateMessage::SetSquadMenu(
                            Some((*self.gui_state.current_cursor_window_point(), squad_ids)),
                        )));
                    } else {
                        messages.push(EngineMessage::GuiState(GuiStateMessage::CenterSceneOn(
                            world_point,
                        )))
                    }
                }
                UIEvent::ImmobileCursorSince(since) => {
                    // Paths to draw if pending order
                    let mut draw_path_findings = vec![];

                    for pending_order in self.gui_state.pending_order() {
                        if since == PENDING_ORDER_PATH_FINDING_DRAW_FRAMES
                            && pending_order.expect_path_finding()
                        {
                            draw_path_findings.push((
                                *pending_order.squad_index(),
                                *pending_order.order_marker_index(),
                                pending_order.cached_points().clone(),
                            ));
                        }
                    }

                    if !draw_path_findings.is_empty() {
                        messages.push(EngineMessage::GuiState(GuiStateMessage::PushUIEvent(
                            UIEvent::DrawPathFinding(draw_path_findings),
                        )));
                    }
                }
                UIEvent::DrawPathFinding(draw_path_findings) => {
                    let mut set_display_paths = vec![];

                    for (squad_id, order_marker_index, cached_points) in draw_path_findings {
                        let (path_mode, start_direction) =
                            self.battle_state.squad_path_mode_and_direction(squad_id);

                        if let Some(world_paths) = self.create_path_finding(
                            squad_id,
                            &order_marker_index,
                            &cached_points,
                            &path_mode,
                            &start_direction,
                        ) {
                            set_display_paths.push(vec![(world_paths, squad_id)]);
                        }
                    }

                    messages.push(EngineMessage::GuiState(GuiStateMessage::SetDisplayPaths(
                        set_display_paths,
                    )));
                }
                UIEvent::CursorMove(_point) => {
                    messages.push(EngineMessage::GuiState(GuiStateMessage::SetDisplayPaths(
                        vec![],
                    )));
                }
                UIEvent::DropSquadTo(squad_index, world_point) => {
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

        if let Some((squad_menu_point, squads)) = self.gui_state.squad_menu() {
            messages.extend(self.digest_squad_menu_select_by_click(
                &point,
                squad_menu_point,
                squads,
            ));
            messages.push(EngineMessage::GuiState(GuiStateMessage::SetSquadMenu(None)));
        } else {
            messages.extend(self.digest_scene_select_by_click(ctx, point));
        }

        // This is a pending order click
        for pending_order in self.gui_state.pending_order() {
            let is_appending = ctx.keyboard.is_key_pressed(VirtualKeyCode::LShift)
                || ctx.keyboard.is_key_pressed(VirtualKeyCode::RShift);

            if is_appending {
                messages.extend(vec![EngineMessage::GuiState(
                    GuiStateMessage::AddCachePointToPendingOrder(
                        self.gui_state.current_cursor_world_point(),
                    ),
                )]);
            } else {
                // If order produced, push it on shared state
                if let Some(order) = self.order_from_pending_order(pending_order) {
                    let squad_leader = self
                        .battle_state
                        .squad(*pending_order.squad_index())
                        .leader();
                    messages.extend(
                        [
                            vec![EngineMessage::PlaySound(Sound::Clac1)],
                            self.define_order(&squad_leader, &order),
                        ]
                        .concat(),
                    )
                } else {
                    messages.push(EngineMessage::PlaySound(Sound::Bip1))
                }

                // In all cases, remove pending order
                messages.extend(vec![EngineMessage::GuiState(
                    GuiStateMessage::SetPendingOrders(vec![]),
                )]);
            }
        }

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
        self.generate_debug_physics(world_point, world_point)
    }

    fn cursor_vector_finished_controlling_soldier(
        &mut self,
        _ctx: &Context,
        start: WindowPoint,
        end: WindowPoint,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        // First try to check if its a missed click in a squad menu
        if let Some((squad_menu_point, squads)) = self.gui_state.squad_menu() {
            if let Some(messages) =
                self.digest_squad_menu_select_by_vector(&start, &end, squad_menu_point, squads)
            {
                // Interrupt vector analyze by returning
                return [
                    messages,
                    vec![EngineMessage::GuiState(GuiStateMessage::SetSquadMenu(None))],
                ]
                .concat();
            }
        }

        if !self.gui_state.pending_order().is_empty() {
            for pending_order in self.gui_state.pending_order() {
                if let Some(order_) = self.order_from_pending_order(pending_order) {
                    let squad_leader = self
                        .battle_state
                        .squad(*pending_order.squad_index())
                        .leader();
                    messages.extend(
                        [
                            vec![EngineMessage::PlaySound(Sound::Clac1)],
                            self.define_order(&squad_leader, &order_),
                        ]
                        .concat(),
                    )
                } else {
                    messages.push(EngineMessage::PlaySound(Sound::Bip1))
                }
                messages.extend(vec![
                    EngineMessage::GuiState(GuiStateMessage::SetPendingOrders(vec![])),
                    EngineMessage::GuiState(GuiStateMessage::SetDisplayPaths(vec![])),
                ]);
            }
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
        if !self.allowed_drop_point(point) {
            return vec![EngineMessage::PlaySound(Sound::Bip1)];
        }

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
        let vehicle = self.battle_state.vehicle(*vehicle_index);
        let grid_point = self.battle_state.map().grid_point_from_world_point(point);
        let vehicle_graphics = VehicleGraphicInfos::from_type(vehicle.type_());
        let chassis_size = vehicle_graphics.size();
        if !self
            .battle_state
            .map()
            .point_allow_vehicle(&grid_point, chassis_size)
        {
            vec![EngineMessage::PlaySound(Sound::Bip1)]
        } else {
            vec![EngineMessage::BattleState(BattleStateMessage::Vehicle(
                *vehicle_index,
                VehicleMessage::SetWorldPosition(*point),
            ))]
        }
    }

    fn drop_pedestrian_squad_to(
        &self,
        squad_index: &SquadUuid,
        point: &WorldPoint,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];
        let squad = self.battle_state.squad(*squad_index);
        let leader = self.battle_state.soldier(squad.leader());
        let cursor_grid_point = self.battle_state.map().grid_point_from_world_point(point);
        let (moves, debug_points) = CoverFinder::new(&self.battle_state, &self.server_config)
            .point(Some(*point))
            .exclude_grid_points(vec![cursor_grid_point])
            .find_arbitrary_cover_points(squad, leader);

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
                    frame_i: self.gui_state.frame_i() + 120,
                    point: new_debug_point.point,
                },
            )))
        }

        messages
    }

    pub fn point_in_border(&self, ctx: &Context, point: &WindowPoint) -> Option<Direction> {
        let (width, height) = ctx.gfx.drawable_size();
        let height = height - HUD_HEIGHT;

        if point.x <= BORDER_SIZE && point.y <= BORDER_SIZE {
            return Some(Direction::NorthWest);
        }
        if point.x >= width - BORDER_SIZE && point.y <= BORDER_SIZE {
            return Some(Direction::NorthEst);
        }
        if point.x >= width - BORDER_SIZE && point.y >= height - BORDER_SIZE {
            return Some(Direction::SouthEst);
        }
        if point.x <= BORDER_SIZE && point.y >= height - BORDER_SIZE {
            return Some(Direction::SouthWest);
        }

        if point.y <= BORDER_SIZE {
            return Some(Direction::North);
        }
        if point.y >= height - BORDER_SIZE {
            return Some(Direction::South);
        }
        if point.x <= BORDER_SIZE {
            return Some(Direction::West);
        }
        if point.x >= width - BORDER_SIZE {
            return Some(Direction::Est);
        }

        None
    }
}
