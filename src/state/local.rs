use glam::Vec2;

use crate::game::Side;
use crate::order::PendingOrder;
use crate::utils::DebugPoint;
use crate::{message::*, types::*};

use crate::debug::{DebugLevel, DebugTerrain};

pub struct LocalState {
    /// Printed frames since start of program
    frame_i: u64,
    /// Side of game instance
    side: Side,
    /// Offset to apply to battle scene by window relative
    pub display_scene_offset: Vec2,
    /// Scale to apply to battle scene by window relative
    pub display_scene_scale: Vec2,
    /// Display or not decor (trees, etc)
    pub draw_decor: bool,
    /// Current debug level to apply
    debug_level: DebugLevel,
    /// Current debug level to apply
    debug_terrain: DebugTerrain,
    /// Current WindowPoint of mouse cursor
    current_cursor_point: WindowPoint,
    /// Last instant since cursor don't move
    last_cursor_move_frame: u64,
    /// WindowPoint where left click was down, if left click currently down
    left_click_down: Option<WindowPoint>,
    /// Vector representing current cursor drag
    current_cursor_vector: Option<(WindowPoint, WindowPoint)>,
    /// Vector of UIEvent (will be consumed)
    ui_events: Vec<UIEvent>,
    /// Selected squad ids
    selected_squads: Vec<SquadUuid>,
    /// Possible currently displayed menu
    squad_menu: Option<(WindowPoint, SquadUuid)>,
    /// Possible current player squad order
    pending_order: Option<(
        PendingOrder,
        SquadUuid,
        Option<OrderMarkerIndex>,
        Vec<WorldPoint>,
    )>, // ..., ..., editing move index, cached points
    /// Paths to display
    display_paths: Vec<(WorldPaths, SquadUuid)>,
    /// Debug points to display if debug mode
    debug_points: Vec<DebugPoint>,
}

impl LocalState {
    pub fn new(side: Side) -> Self {
        Self {
            frame_i: 0,
            side,
            display_scene_offset: Vec2::new(0., 0.),
            // TODO : Zoom is not correctly managed yet
            display_scene_scale: Vec2::new(1., 1.),
            draw_decor: true,
            debug_level: DebugLevel::Debug0,
            debug_terrain: DebugTerrain::None,
            current_cursor_point: WindowPoint::new(0., 0.),
            last_cursor_move_frame: 0,
            left_click_down: None,
            current_cursor_vector: None,
            ui_events: vec![],
            selected_squads: vec![],
            squad_menu: None,
            pending_order: None,
            display_paths: vec![],
            debug_points: vec![],
        }
    }

    pub fn is_first_frame(&self) -> bool {
        self.frame_i == 0
    }

    pub fn get_frame_i(&self) -> u64 {
        self.frame_i
    }

    pub fn increment_frame_i(&mut self) {
        self.frame_i += 1;
    }

    pub fn side(&self) -> &Side {
        &self.side
    }

    pub fn get_debug_level(&self) -> &DebugLevel {
        &self.debug_level
    }

    pub fn get_debug_terrain(&self) -> &DebugTerrain {
        &self.debug_terrain
    }

    pub fn get_current_cursor_window_point(&self) -> &WindowPoint {
        &self.current_cursor_point
    }

    pub fn get_current_cursor_world_point(&self) -> WorldPoint {
        self.world_point_from_window_point(self.current_cursor_point)
    }

    pub fn get_left_click_down_window_point(&self) -> &Option<WindowPoint> {
        &self.left_click_down
    }

    pub fn _get_left_click_down_world_point(&self) -> Option<WorldPoint> {
        if let Some(left_click_down) = self.left_click_down {
            Some(self.world_point_from_window_point(left_click_down))
        } else {
            None
        }
    }

    pub fn current_cursor_vector_window_points(&self) -> &Option<(WindowPoint, WindowPoint)> {
        &self.current_cursor_vector
    }

    pub fn _current_cursor_vector_world_points(&self) -> Option<(WorldPoint, WorldPoint)> {
        if let Some((start, end)) = self.current_cursor_vector {
            let world_start = self.world_point_from_window_point(start);
            let world_end = self.world_point_from_window_point(end);
            Some((world_start, world_end))
        } else {
            None
        }
    }

    pub fn pop_ui_event(&mut self) -> Option<UIEvent> {
        self.ui_events.pop()
    }

    pub fn world_point_from_window_point(&self, window_point: WindowPoint) -> WorldPoint {
        WorldPoint::from(
            window_point.apply(-self.display_scene_offset).to_vec2() / self.display_scene_scale,
        )
    }

    pub fn window_point_from_world_point(&self, world_point: WorldPoint) -> WindowPoint {
        WindowPoint::from(
            world_point.apply(self.display_scene_offset).to_vec2() * self.display_scene_scale,
        )
    }

    pub fn selected_squads(&self) -> &[SquadUuid] {
        &self.selected_squads
    }

    pub fn get_squad_menu(&self) -> &Option<(WindowPoint, SquadUuid)> {
        &self.squad_menu
    }

    pub fn get_last_cursor_move_frame(&self) -> u64 {
        self.last_cursor_move_frame
    }

    pub fn get_pending_order(
        &self,
    ) -> &Option<(
        PendingOrder,
        SquadUuid,
        Option<OrderMarkerIndex>,
        Vec<WorldPoint>,
    )> {
        &self.pending_order
    }

    pub fn get_display_paths(&self) -> &[(WorldPaths, SquadUuid)] {
        &self.display_paths
    }

    pub fn debug_points_mut(&mut self) -> &mut Vec<DebugPoint> {
        &mut self.debug_points
    }

    pub fn set_debug_points(&mut self, debug_points: Vec<DebugPoint>) {
        self.debug_points = debug_points
    }

    pub fn react(&mut self, local_state_message: LocalStateMessage) {
        match local_state_message {
            LocalStateMessage::SetCursorPoint(point) => {
                //
                self.current_cursor_point = point;
                self.last_cursor_move_frame = self.frame_i;
            }
            LocalStateMessage::SetSceneDisplayOffset(offset) => {
                //
                self.display_scene_offset += offset.to_vec2();
            }
            LocalStateMessage::SetDebugLevel(level) => {
                //
                self.debug_level = level;
            }
            LocalStateMessage::SetDebugTerrain(value) => {
                //
                self.debug_terrain = value;
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
            LocalStateMessage::SetSelectedSquads(selected_squads) => {
                //
                self.selected_squads = selected_squads
            }
            LocalStateMessage::SetSquadMenu(squad_menu) => {
                //
                self.squad_menu = squad_menu
            }
            LocalStateMessage::SetPendingOrder(pending_order) => {
                //
                self.pending_order = pending_order
            }
            LocalStateMessage::SetDisplayPaths(display_paths) => {
                //
                self.display_paths = display_paths
            }
            LocalStateMessage::AddCachePointToPendingOrder(new_point) => {
                let (_, _, _, cached_points) = self
                    .pending_order
                    .as_mut()
                    .expect("Add cache point imply existing pending order");
                cached_points.push(new_point)
            }
            LocalStateMessage::PushDebugPoint(debug_point) => {
                //
                self.debug_points.push(debug_point)
            }
        }
    }
}
