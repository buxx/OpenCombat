use ggez::{
    event::{KeyCode, KeyMods, MouseButton},
    input, Context,
};

use crate::{
    debug::{DebugLevel, DebugPhysics, DebugTerrain},
    message::*,
    types::*,
};
use serde::{Deserialize, Serialize};

use super::Engine;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Control {
    Soldiers,
    Map,
    Physics,
}

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
                LocalStateMessage::ApplyOnSceneDisplayOffset(Offset::new(
                    move_mode.to_pixels_offset(),
                    0.,
                )),
            ));
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Right) {
            messages.push(Message::LocalState(
                LocalStateMessage::ApplyOnSceneDisplayOffset(Offset::new(
                    -move_mode.to_pixels_offset(),
                    0.,
                )),
            ));
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Up) {
            messages.push(Message::LocalState(
                LocalStateMessage::ApplyOnSceneDisplayOffset(Offset::new(
                    0.,
                    move_mode.to_pixels_offset(),
                )),
            ));
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Down) {
            messages.push(Message::LocalState(
                LocalStateMessage::ApplyOnSceneDisplayOffset(Offset::new(
                    0.,
                    -move_mode.to_pixels_offset(),
                )),
            ));
        }

        messages
    }

    pub fn collect_key_pressed(
        &self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) -> Vec<Message> {
        let mut messages = vec![];

        if keycode == KeyCode::LControl || keycode == KeyCode::RControl {
            messages.push(Message::LocalState(LocalStateMessage::SetControl(
                Control::Map,
            )))
        }

        messages
    }

    pub fn collect_key_released(&self, _ctx: &mut Context, keycode: KeyCode) -> Vec<Message> {
        let mut messages = vec![];

        match keycode {
            KeyCode::F12 => {
                let new_debug_level = match self.local_state.get_debug_level() {
                    DebugLevel::Debug0 => DebugLevel::Debug1,
                    DebugLevel::Debug1 => DebugLevel::Debug2,
                    DebugLevel::Debug2 => DebugLevel::Debug3,
                    DebugLevel::Debug3 => DebugLevel::Debug0,
                };
                messages.push(Message::LocalState(LocalStateMessage::SetDebugLevel(
                    new_debug_level,
                )));
            }
            KeyCode::F11 => {
                let new_debug_terrain = match self.local_state.get_debug_terrain() {
                    DebugTerrain::None => DebugTerrain::Opacity,
                    DebugTerrain::Opacity => DebugTerrain::Tiles,
                    DebugTerrain::Tiles => DebugTerrain::None,
                };
                messages.push(Message::LocalState(LocalStateMessage::SetDebugTerrain(
                    new_debug_terrain,
                )));
            }
            KeyCode::F10 => {
                let new_debug_physics = match self.local_state.get_debug_physics() {
                    DebugPhysics::None => DebugPhysics::MosinNagantM1924GunFire,
                    DebugPhysics::MosinNagantM1924GunFire => DebugPhysics::BrandtMle2731Shelling,
                    DebugPhysics::BrandtMle2731Shelling => DebugPhysics::None,
                };
                println!("Debug physics : {:?}", &new_debug_physics);
                if &new_debug_physics != &DebugPhysics::None {
                    messages.push(Message::LocalState(LocalStateMessage::SetControl(
                        Control::Physics,
                    )));
                } else {
                    messages.push(Message::LocalState(LocalStateMessage::SetControl(
                        Control::Soldiers,
                    )));
                }
                messages.push(Message::LocalState(LocalStateMessage::SetDebugPhysics(
                    new_debug_physics,
                )));
            }
            KeyCode::F9 => messages.push(Message::LocalState(LocalStateMessage::ChangeSide)),
            KeyCode::LControl | KeyCode::RControl => messages.push(Message::LocalState(
                LocalStateMessage::SetControl(self.determine_controlling()),
            )),
            _ => {}
        };

        messages
    }

    pub fn determine_controlling(&self) -> Control {
        match self.local_state.get_debug_physics() {
            DebugPhysics::None => Control::Soldiers,
            DebugPhysics::MosinNagantM1924GunFire | DebugPhysics::BrandtMle2731Shelling => {
                Control::Physics
            }
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
                if self.local_state.is_controlling(&Control::Map) {
                    let last_cursor_point = self.local_state.get_current_cursor_window_point();
                    messages.push(Message::LocalState(
                        LocalStateMessage::ApplyOnSceneDisplayOffset(Offset::from_vec2(
                            cursor_point.to_vec2() - last_cursor_point.to_vec2(),
                        )),
                    ))
                }
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
                if self.local_state.is_controlling(&Control::Soldiers) {
                    for (_, order_marker, squad_id, world_point, order_marker_i) in
                        self.shared_state.order_markers(self.local_state.side())
                    {
                        let window_point =
                            self.local_state.window_point_from_world_point(world_point);
                        // FIXME : Must take angle (see v1)
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

    pub fn collect_mouse_wheel(&self, _ctx: &mut Context, _x: f32, y: f32) -> Vec<Message> {
        let mut messages = vec![];

        messages.push(Message::LocalState(LocalStateMessage::ScaleUpdate(
            y / 10.0,
        )));

        messages
    }
}
