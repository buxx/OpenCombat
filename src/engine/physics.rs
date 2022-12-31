use ggez::{graphics::MeshBuilder, GameResult};

use crate::{
    message::{LocalStateMessage, Message},
    physics::event::{bullet::BulletFire, explosion::Explosion},
    types::{BulletFireIndex, ExplosionIndex},
    utils::GREY,
    NetworkMode,
};

use super::Engine;

impl Engine {
    pub fn tick_physics(&self) -> Vec<Message> {
        let mut messages = vec![];

        if self.local_state.get_frame_i() % self.config.physics_update_freq() == 0 {
            messages.extend(self.tick_bullet_fires());
            messages.extend(self.tick_explosions());
        }

        messages
    }

    pub fn tick_bullet_fires(&self) -> Vec<Message> {
        let mut messages = vec![];
        let frame_i = self.local_state.get_frame_i();

        for (i, bullet_fire) in self.local_state.bullet_fires().iter().enumerate() {
            messages.extend(bullet_fire.fx(frame_i));

            if self.config.network_mode() == &NetworkMode::Server && bullet_fire.effective(frame_i)
            {
                messages.extend(self.bullet_fire_effects(bullet_fire))
            }

            if bullet_fire.finished(frame_i) {
                messages.push(Message::LocalState(LocalStateMessage::RemoveBulletFire(
                    BulletFireIndex(i),
                )))
            }
        }

        messages
    }

    fn bullet_fire_effects(&self, bullet_fire: &BulletFire) -> Vec<Message> {
        let mut messages = vec![];

        messages
    }

    pub fn tick_explosions(&self) -> Vec<Message> {
        let mut messages = vec![];
        let frame_i = self.local_state.get_frame_i();

        for (i, explosion) in self.local_state.explosions().iter().enumerate() {
            messages.extend(explosion.fx(self.local_state.get_frame_i()));

            if self.config.network_mode() == &NetworkMode::Server && explosion.effective(frame_i) {
                messages.extend(self.explosion_effects(explosion))
            }

            if explosion.finished(frame_i) {
                messages.push(Message::LocalState(LocalStateMessage::RemoveExplosion(
                    ExplosionIndex(i),
                )))
            }
        }

        messages
    }

    fn explosion_effects(&self, explosion: &Explosion) -> Vec<Message> {
        let mut messages = vec![];

        messages
    }

    pub fn draw_physics(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        self.draw_bullet_fires(mesh_builder)?;
        self.draw_explosions(mesh_builder)?;

        Ok(())
    }

    pub fn draw_bullet_fires(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        for bullet_fire in self.local_state.bullet_fires() {
            let from = self
                .local_state
                .window_point_from_world_point(*bullet_fire.from());
            let to = self
                .local_state
                .window_point_from_world_point(*bullet_fire.to());
            mesh_builder.line(&vec![from.to_vec2(), to.to_vec2()], 1.0, GREY)?;
        }
        Ok(())
    }

    pub fn draw_explosions(&self, _mesh_builder: &mut MeshBuilder) -> GameResult {
        // Nothing here because drawn by graphics sequences
        Ok(())
    }
}
