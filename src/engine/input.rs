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

        // Useful for actions expecting cursor immobilization
        let cursor_immobile_since =
            self.local_state.get_frame_i() - self.local_state.get_last_cursor_move_frame();
        messages.push(Message::LocalState(LocalStateMessage::PushUIEvent(
            UIEvent::ImmobileCursorSince(cursor_immobile_since),
        )));

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

        messages
    }

    pub fn key_released(&self, _ctx: &mut Context, keycode: KeyCode) -> Vec<Message> {
        match keycode {
            KeyCode::F12 => {
                let new_debug_level = match self.local_state.get_debug() {
                    DebugLevel::Debug0 => DebugLevel::Debug1,
                    DebugLevel::Debug1 => DebugLevel::Debug2,
                    DebugLevel::Debug2 => DebugLevel::Debug3,
                    DebugLevel::Debug3 => DebugLevel::Debug0,
                };
                vec![Message::LocalState(LocalStateMessage::SetDebugLevel(
                    new_debug_level,
                ))]
            }
            _ => vec![],
        }
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

        // Update cursor position at each frames
        messages.extend(vec![
            Message::LocalState(LocalStateMessage::SetCursorPoint(cursor_point)),
            Message::LocalState(LocalStateMessage::PushUIEvent(UIEvent::CursorMove(
                cursor_point,
            ))),
        ]);

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
            MouseButton::Left => {
                // Update cursor down position
                messages.push(Message::LocalState(LocalStateMessage::SetLeftClickDown(
                    Some(WindowPoint::new(x, y)),
                )));

                // Check if any order under the cursor
                for (_, order_marker, squad_id, world_point, order_marker_i) in
                    self.shared_state.order_markers()
                {
                    let window_point = self.local_state.window_point_from_world_point(world_point);
                    if order_marker
                        .sprite_info()
                        .contains(&window_point, &WindowPoint::new(x, y))
                    {
                        messages.push(Message::LocalState(LocalStateMessage::SetPendingOrder(
                            Some((
                                order_marker.to_pending_order(),
                                squad_id,
                                Some(order_marker_i),
                                vec![],
                            )),
                        )));
                    }
                }
            }
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
            MouseButton::Right => {
                messages.push(Message::LocalState(LocalStateMessage::PushUIEvent(
                    UIEvent::FinishedCursorRightClick(WindowPoint::new(x, y)),
                )));
            }
            _ => {}
        }

        messages
    }
}
