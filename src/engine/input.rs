use ggez::{event::KeyCode, input, Context};
use glam::Vec2;

use crate::{behavior::Behavior, message::*, order::Order, types::*};

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
                            messages.push(Message::State(StateMessage::PushOrder(
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
            messages.push(Message::Engine(EngineMessage::ApplySceneDisplayOffset(
                Offset::new(move_mode.to_pixels_offset(), 0.),
            )));
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Right) {
            messages.push(Message::Engine(EngineMessage::ApplySceneDisplayOffset(
                Offset::new(-move_mode.to_pixels_offset(), 0.),
            )));
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Up) {
            messages.push(Message::Engine(EngineMessage::ApplySceneDisplayOffset(
                Offset::new(0., move_mode.to_pixels_offset()),
            )));
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Down) {
            messages.push(Message::Engine(EngineMessage::ApplySceneDisplayOffset(
                Offset::new(0., -move_mode.to_pixels_offset()),
            )));
        }

        messages
    }
}
