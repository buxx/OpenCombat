use battle_core::physics::event::bullet::BulletFire;
use ggez::graphics::MeshBuilder;
use ggez::GameResult;

use crate::engine::Engine;

use crate::{engine::message::EngineMessage, utils::GREY};

impl Engine {
    pub fn tick_bullet_fires(&self) -> Vec<EngineMessage> {
        puffin::profile_scope!("tick_bullet_fires");
        let mut messages = vec![];

        for bullet_fire in self.battle_state.bullet_fires() {
            messages.extend(self.bullet_fire_fx(bullet_fire));
        }

        messages
    }

    pub fn bullet_fire_fx(&self, bullet_fire: &BulletFire) -> Vec<EngineMessage> {
        let mut messages = vec![];

        if bullet_fire.start() == self.gui_state.get_frame_i() {
            for sound in bullet_fire.gun_fire_sound_type().fire_sounds() {
                messages.push(EngineMessage::PlaySound(sound.clone()));
            }
        }

        messages
    }

    pub fn draw_bullet_fires(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for bullet_fire in self.battle_state.bullet_fires() {
            let from = self
                .gui_state
                .window_point_from_world_point(*bullet_fire.from());
            let to = self
                .gui_state
                .window_point_from_world_point(*bullet_fire.to());
            mesh_builder.line(&vec![from.to_vec2(), to.to_vec2()], 1.0, GREY)?;
        }
        Ok(())
    }
}
