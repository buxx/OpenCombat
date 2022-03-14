use ggez::{
    event::{KeyCode, MouseButton},
    input, Context,
};
use glam::Vec2;

use crate::{behavior::Behavior, debug::DebugLevel, message::*, order::Order, types::*};

use super::Engine;

enum MoveScreenMode {
    Normal,
    Speed,
}

impl MoveScreenMode {
    pub fn to_pixels_offset(&self) -> f32 {
        match self {
            MoveScreenMode::Normal => 2.0,
            MoveScreenMode::Speed => 15.0,
        }
    }
}

impl Engine {
    pub fn collect_player_inputs(&self, ctx: &mut Context) -> Vec<Message> {
        let mut messages = vec![];

        messages.extend(self.collect_keyboard_inputs(ctx));

        // TODO : hardcode code for test purposes
        if self.local_state.frame_i == 60 {
            for (i, entity) in self.shared_state.entities().iter().enumerate() {
                if self.entity_is_squad_leader(EntityIndex(i)) {
                    match entity.get_behavior() {
                        Behavior::Idle => {
                            let a1 = entity.get_world_point().apply(Vec2::new(10., 0.));
                            let a2 = entity.get_world_point().apply(Vec2::new(0., 10.));
                            let b1 = entity.get_world_point().apply(Vec2::new(20., 10.));
                            let b2 = entity.get_world_point().apply(Vec2::new(10., 20.));
                            let a = WorldPath::new(vec![a1, a2]);
                            let b = WorldPath::new(vec![b1, b2]);
                            messages.push(Message::SharedState(SharedStateMessage::PushOrder(
                                entity.squad_uuid(),
                                Order::MoveTo(WorldPaths::new(vec![a, b])),
                            )));
                        }
                        _ => {}
                    }
                }
            }
        }

        messages
    }

    fn collect_keyboard_inputs(&self, ctx: &mut Context) -> Vec<Message> {
        let mut messages = vec![];

        let shift_pressed = input::keyboard::is_key_pressed(ctx, KeyCode::LShift)
            || input::keyboard::is_key_pressed(ctx, KeyCode::RShift);
        let move_mode = if shift_pressed {
            MoveScreenMode::Speed
        } else {
            MoveScreenMode::Normal
        };

        // Move battle scene on the window according to user keys
        if input::keyboard::is_key_pressed(ctx, KeyCode::Left) {
            messages.push(Message::LocalState(
                LocalStateMessage::SetSceneDisplayOffset(Offset::new(
                    move_mode.to_pixels_offset(),
                    0.,
                )),
            ));
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Right) {
            messages.push(Message::LocalState(
                LocalStateMessage::SetSceneDisplayOffset(Offset::new(
                    -move_mode.to_pixels_offset(),
                    0.,
                )),
            ));
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Up) {
            messages.push(Message::LocalState(
                LocalStateMessage::SetSceneDisplayOffset(Offset::new(
                    0.,
                    move_mode.to_pixels_offset(),
                )),
            ));
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Down) {
            messages.push(Message::LocalState(
                LocalStateMessage::SetSceneDisplayOffset(Offset::new(
                    0.,
                    -move_mode.to_pixels_offset(),
                )),
            ));
        }

        // Debug
        if input::keyboard::is_key_pressed(ctx, KeyCode::F12)
            && !input::keyboard::is_key_repeated(ctx)
        {
            let new_debug_level = match self.local_state.get_debug() {
                DebugLevel::Debug0 => DebugLevel::Debug1,
                DebugLevel::Debug1 => DebugLevel::Debug2,
                DebugLevel::Debug2 => DebugLevel::Debug3,
                DebugLevel::Debug3 => DebugLevel::Debug0,
            };
            messages.push(Message::LocalState(LocalStateMessage::SetDebugLevel(
                new_debug_level,
            )));
        }

        messages
    }

    pub fn collect_mouse_motion(
        &self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Vec<Message> {
        let mut messages = vec![];

        // Instant information
        let cursor_point = WindowPoint::new(x, y);
        // let cursor_world_point = WorldPoint::from(
        //     cursor_window_point
        //         .apply(self.local_state.display_scene_offset)
        //         .to_vec2()
        //         / self.local_state.display_scene_scale,
        // );

        // let new_current_cursor_position = WindowPoint::new(x, y);
        // let new_current_grid_cursor_position = grid_point_from_scene_point(
        //     &scene_point_from_window_point(
        //         &new_current_cursor_position,
        //         &self.display_offset,
        //         self.scale,
        //     ),
        //     &self.map,
        // );

        // if self.current_cursor_point != new_current_cursor_position {
        //     self.user_events
        //         .push(UserEvent::CursorMove(new_current_cursor_position));
        //     self.current_cursor_point = new_current_cursor_position;
        // };

        // if self.current_cursor_grid_point != new_current_grid_cursor_position {
        //     self.current_cursor_grid_point = new_current_grid_cursor_position;
        //     self.current_prepare_move_found_paths = HashMap::new();
        //     self.cursor_on_same_grid_point_since = Instant::now();
        // } else {
        //     self.cursor_on_same_grid_point_since = Instant::now();
        // }

        // if self.dragging.is_some() {
        //     self.user_events.push(UserEvent::MoveDrag)
        // }

        // Update cursor position at each frames
        messages.push(Message::LocalState(LocalStateMessage::SetCursorPoint(
            cursor_point,
        )));

        if let Some(left_click_down) = self.local_state.get_left_click_down_window_point() {
            if left_click_down != &cursor_point {
                messages.push(Message::LocalState(
                    LocalStateMessage::SetCurrentCursorVector(Some((
                        *left_click_down,
                        cursor_point,
                    ))),
                ));
            }
        }

        messages
    }

    pub fn collect_mouse_down(
        &self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Vec<Message> {
        let mut messages = vec![];

        match button {
            MouseButton::Left => messages.push(Message::LocalState(
                LocalStateMessage::SetLeftClickDown(Some(WindowPoint::new(x, y))),
            )),
            _ => {}
        }

        messages
    }

    pub fn collect_mouse_up(
        &self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Vec<Message> {
        let mut messages = vec![];

        match button {
            MouseButton::Left => {
                let start_point = self
                    .local_state
                    .get_left_click_down_window_point()
                    .expect("No left button down before button up ?!");
                let end_point = WindowPoint::new(x, y);

                // No more longer left click down or current drag
                messages.extend(vec![
                    Message::LocalState(LocalStateMessage::SetLeftClickDown(None)),
                    Message::LocalState(LocalStateMessage::SetCurrentCursorVector(None)),
                ]);

                // Determine if it is a simple click or a drag
                if start_point != end_point {
                    messages.push(Message::LocalState(LocalStateMessage::PushUIEvent(
                        UIEvent::FinishedCursorVector(start_point, end_point),
                    )));
                } else {
                    messages.push(Message::LocalState(LocalStateMessage::PushUIEvent(
                        UIEvent::FinishedCursorLeftClick(end_point),
                    )));
                }
            }
            _ => {}
        }

        messages
    }
}
