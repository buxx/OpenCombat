use ggez::graphics::MeshBuilder;
use ggez::GameResult;

use crate::engine::Engine;

use crate::{
    message::{LocalStateMessage, Message},
    physics::event::bullet::BulletFire,
    types::BulletFireIndex,
    utils::GREY,
    NetworkMode,
};

impl Engine {
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
}
