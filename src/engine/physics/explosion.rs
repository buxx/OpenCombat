use ggez::graphics::MeshBuilder;
use ggez::GameResult;

use crate::engine::Engine;

use crate::message::SharedStateMessage;
use crate::physics::effect::Effect;
use crate::physics::utils::meters_between_scene_points;
use crate::{
    message::{LocalStateMessage, Message},
    physics::event::explosion::Explosion,
    types::ExplosionIndex,
    NetworkMode,
};

impl Engine {
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

    // FIXME : find algorithm kill/injure about explosives + terrain + position
    fn explosion_effects(&self, explosion: &Explosion) -> Vec<Message> {
        let mut messages = vec![];
        let point = explosion.point();
        let explosive_type = explosion.type_();
        let _blast = explosive_type.blast();

        for soldier in self.shared_state.soldiers() {
            if !soldier.can_feel_explosion() {
                continue;
            }

            let distance_from_point =
                meters_between_scene_points(&soldier.get_world_point(), point);

            if distance_from_point.0 < 4.0 {
                messages.push(Message::SharedState(SharedStateMessage::PushPhysicsEffect(
                    Effect::KillingBlast(soldier.uuid()),
                )))
            } else if distance_from_point.0 < 7.0 {
                messages.push(Message::SharedState(SharedStateMessage::PushPhysicsEffect(
                    Effect::StunningBlast(soldier.uuid()),
                )))
            } else if distance_from_point.0 < 75.0 {
                messages.push(Message::SharedState(SharedStateMessage::PushPhysicsEffect(
                    Effect::ProximityBlast(soldier.uuid(), distance_from_point.clone()),
                )))
            }
        }

        messages
    }

    pub fn draw_explosions(&self, _mesh_builder: &mut MeshBuilder) -> GameResult {
        // Nothing here because drawn by graphics sequences
        Ok(())
    }
}
