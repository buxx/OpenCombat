use ggez::{
    graphics::{DrawMode, MeshBuilder, Rect, StrokeOptions},
    GameResult,
};

use crate::{
    config::{DEFAULT_SELECTED_SQUARE_SIDE, DEFAULT_SELECTED_SQUARE_SIDE_HALF},
    message::*,
    types::*,
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

                    if !squad_menu_displayed {
                        messages.extend(self.digest_scene_select_by_click(point));
                    } else {
                        messages.push(Message::LocalState(LocalStateMessage::SetSquadMenu(None)));
                    }
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
            }
        }

        messages
    }

    fn digest_scene_select_by_click(&self, point: WindowPoint) -> Vec<Message> {
        let world_point = self.local_state.world_point_from_window_point(point);
        let entity_indexes = self.get_side_entities_at_point(world_point);
        if entity_indexes.len() > 0 {
            let squad_ids = self.squad_ids_from_entities(vec![entity_indexes[0]]);
            vec![Message::LocalState(LocalStateMessage::SetSelectedSquads(
                squad_ids,
            ))]
        } else {
            vec![Message::LocalState(LocalStateMessage::SetSelectedSquads(
                vec![],
            ))]
        }
    }
}
