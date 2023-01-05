use ggez::graphics::MeshBuilder;
use ggez::GameResult;
use rand::Rng;

use crate::engine::Engine;

use crate::message::SharedStateMessage;
use crate::physics::effect::Effect;
use crate::physics::utils::meters_between_scene_points;
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

    // FIXME : find algorithm kill/injure about bullet + terrain + position
    fn bullet_fire_effects(&self, bullet_fire: &BulletFire) -> Vec<Message> {
        let mut messages = vec![];
        let point = bullet_fire.point();
        let _weapon = bullet_fire.weapon();

        for soldier in self.shared_state.soldiers() {
            if !soldier.can_feel_bullet_fire() {
                continue;
            }

            let distance_from_point =
                meters_between_scene_points(&soldier.get_world_point(), point);

            if distance_from_point.0 < 0.5 {
                let mut rng = rand::thread_rng();
                let value: u8 = rng.gen();
                if value < 10 {
                    messages.push(Message::SharedState(SharedStateMessage::PushPhysicsEffect(
                        Effect::KillingBullet(soldier.uuid()),
                    )))
                } else if value < 50 {
                    messages.push(Message::SharedState(SharedStateMessage::PushPhysicsEffect(
                        Effect::InjuringBullet(soldier.uuid()),
                    )))
                } else {
                    messages.push(Message::SharedState(SharedStateMessage::PushPhysicsEffect(
                        Effect::ProximityBullet(soldier.uuid(), distance_from_point.clone()),
                    )))
                }
            } else {
                messages.push(Message::SharedState(SharedStateMessage::PushPhysicsEffect(
                    Effect::ProximityBullet(soldier.uuid(), distance_from_point.clone()),
                )))
            }
        }

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
