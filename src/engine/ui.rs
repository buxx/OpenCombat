use ggez::{
    graphics::{Color, DrawMode, MeshBuilder, Rect, StrokeOptions},
    GameResult,
};
use glam::Vec2;

use crate::{
    config::{
        DEFAULT_SELECTED_SQUARE_SIDE, DEFAULT_SELECTED_SQUARE_SIDE_HALF,
        PENDING_ORDER_PATH_FINDING_DRAW_FRAMES,
    },
    message::*,
    physics::path::find_path,
    types::*,
    ui::menu::squad_menu_sprite_info,
    utils::GREEN,
};

use super::Engine;

impl Engine {
    pub fn generate_selected_entities_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for squad_uuid in self.local_state.selected_squads() {
            for entity_index in self.shared_state.squad(*squad_uuid).members() {
                let entity = self.shared_state.entity(*entity_index);
                let point = self
                    .local_state
                    .window_point_from_world_point(entity.get_world_point());
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

    pub fn get_side_entities_at_point(&self, point: WorldPoint) -> Vec<EntityIndex> {
        let entity_indexes = self.get_entities_at_point(point);
        self.filter_entities_by_side(entity_indexes)
    }

    pub fn ui_events(&mut self) -> Vec<Message> {
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

                    // In all cases, clean some things
                    messages.push(Message::LocalState(LocalStateMessage::SetDisplayPaths(
                        vec![],
                    )));
                }
                UIEvent::FinishedCursorVector(start, end) => {
                    // TODO : Do this if not currently dragging
                    let world_start = self.local_state.world_point_from_window_point(start);
                    let world_end = self.local_state.world_point_from_window_point(end);
                    let entity_indexes = self.get_entities_in_area(world_start, world_end);
                    let entity_indexes = self.filter_entities_by_side(entity_indexes);
                    let squad_ids = self.squad_ids_from_entities(entity_indexes);
                    messages.push(Message::LocalState(LocalStateMessage::SetSelectedSquads(
                        squad_ids,
                    )));
                }
                UIEvent::FinishedCursorRightClick(point) => {
                    let world_point = self.local_state.world_point_from_window_point(point);
                    let entity_indexes = self.get_side_entities_at_point(world_point);
                    let mut squad_id: Option<SquadUuid> = None;

                    // If squad under cursor, select it
                    if entity_indexes.len() > 0 {
                        let squad_id_ = self.squad_ids_from_entities(entity_indexes.clone())[0];
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
                    if let Some((pending_order, squad_id)) = self.local_state.get_pending_order() {
                        if since == PENDING_ORDER_PATH_FINDING_DRAW_FRAMES {
                            if pending_order.expect_path_finding() {
                                messages.push(Message::LocalState(LocalStateMessage::PushUIEvent(
                                    UIEvent::DrawPathFinding(*squad_id),
                                )));
                            }
                        }
                    }
                }
                UIEvent::DrawPathFinding(quad_id) => {
                    let squad = self.shared_state.squad(quad_id);
                    let entity = self.shared_state.entity(squad.leader());
                    let entity_world_point = entity.get_world_point();
                    let entity_grid_point = self.grid_point_from_world_point(entity_world_point);
                    let cursor_world_point = self.local_state.get_current_cursor_world_point();
                    let cursor_grid_point = self.grid_point_from_world_point(cursor_world_point);

                    let grid_point_path =
                        find_path(&self.map, &entity_grid_point, &cursor_grid_point)
                            .unwrap_or(vec![]);
                    if grid_point_path.len() > 0 {
                        let world_point_path = grid_point_path
                            .iter()
                            .map(|p| self.world_point_from_grid_point(GridPoint::from(*p)))
                            .collect();
                        let world_path = WorldPath::new(world_point_path);
                        let world_paths = WorldPaths::new(vec![world_path]);
                        messages.push(Message::LocalState(LocalStateMessage::SetDisplayPaths(
                            vec![world_paths],
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

    fn digest_scene_select_by_click(&self, point: WindowPoint) -> Vec<Message> {
        let world_point = self.local_state.world_point_from_window_point(point);
        let entity_indexes = self.get_side_entities_at_point(world_point);
        if entity_indexes.len() > 0 {
            let squad_ids = self.squad_ids_from_entities(vec![entity_indexes[0]]);
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
                Some((menu_item.to_pending_order(), squad_id)),
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
}
