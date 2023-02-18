use battle_core::physics::event::explosion::Explosion;
use ggez::graphics::MeshBuilder;
use ggez::GameResult;

use crate::engine::message::EngineMessage;
use crate::engine::Engine;
use crate::graphics::message::GraphicsMessage;

impl Engine {
    pub fn tick_explosions(&self) -> Vec<EngineMessage> {
        puffin::profile_scope!("tick_explosions");
        let mut messages = vec![];
        let frame_i = self.gui_state.get_frame_i();

        for explosion in self.battle_state.explosions() {
            messages.extend(self.explosion_fx(explosion));

            if explosion.start() == frame_i {
                messages.push(EngineMessage::Graphics(
                    GraphicsMessage::PushExplosionAnimation(
                        explosion.point().clone(),
                        explosion.explosive_type().clone(),
                    ),
                ))
            }

            if explosion.finished(frame_i) {
                // TODO : Remove by self.point can remove other explosions. Find better methodology
                messages.push(EngineMessage::Graphics(
                    GraphicsMessage::RemoveExplosionAnimation(explosion.point().clone()),
                ))
            }
        }

        messages
    }

    pub fn explosion_fx(&self, explosion: &Explosion) -> Vec<EngineMessage> {
        let mut messages = vec![];

        if explosion.start() == self.gui_state.get_frame_i() {
            for sound in explosion.explosive_type().sounds() {
                messages.push(EngineMessage::PlaySound(sound.clone()));
            }

            messages.push(EngineMessage::Graphics(
                GraphicsMessage::PushExplosionAnimation(
                    explosion.point().clone(),
                    explosion.explosive_type().clone(),
                ),
            ));
        }

        messages
    }

    pub fn draw_explosions(&self, _mesh_builder: &mut MeshBuilder) -> GameResult {
        // Nothing here because drawn by graphics sequences
        Ok(())
    }
}
