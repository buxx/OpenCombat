use glam::Vec2;

use crate::{message::*, types::*};

use crate::debug::DebugLevel;

pub struct LocalState {
    /// Printed frames since start of program
    pub frame_i: u64,
    /// Offset to apply to battle scene by window relative
    pub display_scene_offset: Vec2,
    /// Scale to apply to battle scene by window relative
    pub display_scene_scale: Vec2,
    /// Display or not decor (trees, etc)
    pub draw_decor: bool,
    /// List of selected Squads
    pub selected_squads: Vec<SquadUuid>,
    /// Current debug level to apply
    debug: DebugLevel,
    /// Current WindowPoint of mouse cursor
    current_cursor_point: WindowPoint,
    /// WindowPoint where left click was down, if left click currently down
    left_click_down: Option<WindowPoint>,
    /// Vector representing current cursor drag
    current_cursor_vector: Option<(WindowPoint, WindowPoint)>,
    /// Vector of UIEvent (will be consumed)
    ui_events: Vec<UIEvent>,
}

impl LocalState {
    pub fn new() -> Self {
        Self {
            frame_i: 0,
            display_scene_offset: Vec2::new(0., 0.),
            display_scene_scale: Vec2::new(1., 1.),
            draw_decor: true,
            selected_squads: vec![],
            debug: DebugLevel::Debug0,
            current_cursor_point: WindowPoint::new(0., 0.),
            left_click_down: None,
            current_cursor_vector: None,
            ui_events: vec![],
        }
    }

    pub fn get_debug(&self) -> &DebugLevel {
        &self.debug
    }

    pub fn get_current_cursor_window_point(&self) -> &WindowPoint {
        &self.current_cursor_point
    }

    pub fn get_current_cursor_world_point(&self) -> WorldPoint {
        WorldPoint::from(
            self.current_cursor_point
                .apply(-self.display_scene_offset)
                .to_vec2()
                / self.display_scene_scale,
        )
    }

    pub fn get_left_click_down_window_point(&self) -> &Option<WindowPoint> {
        &self.left_click_down
    }

    pub fn get_left_click_down_world_point(&self) -> Option<WorldPoint> {
        if let Some(left_click_down) = self.left_click_down {
            Some(WorldPoint::from(
                left_click_down.apply(-self.display_scene_offset).to_vec2()
                    / self.display_scene_scale,
            ))
        } else {
            None
        }
    }

    pub fn current_cursor_vector_window_points(&self) -> &Option<(WindowPoint, WindowPoint)> {
        &self.current_cursor_vector
    }

    pub fn current_cursor_vector_world_points(&self) -> Option<(WorldPoint, WorldPoint)> {
        if let Some((start, end)) = self.current_cursor_vector {
            let start = start.apply(-self.display_scene_offset);
            let end = end.apply(-self.display_scene_offset);
            let start = start.to_vec2() / self.display_scene_scale;
            let end = end.to_vec2() / self.display_scene_scale;
            Some((WorldPoint::from(start), WorldPoint::from(end)))
        } else {
            None
        }
    }

    pub fn react(&mut self, local_state_message: LocalStateMessage) {
        match local_state_message {
            LocalStateMessage::SetCursorPoint(point) => {
                //
                self.current_cursor_point = point;
            }
            LocalStateMessage::SetSceneDisplayOffset(offset) => {
                //
                self.display_scene_offset += offset.to_vec2();
            }
            LocalStateMessage::SetDebugLevel(level) => {
                //
                self.debug = level;
            }
            LocalStateMessage::SetLeftClickDown(point) => {
                //
                self.left_click_down = point
            }
            LocalStateMessage::SetCurrentCursorVector(vector) => {
                //
                self.current_cursor_vector = vector
            }
            LocalStateMessage::PushUIEvent(event) => {
                //
                self.ui_events.push(event)
            }
        }
    }
}
