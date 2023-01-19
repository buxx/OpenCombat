use ggez::graphics::MeshBuilder;
use ggez::GameResult;

use crate::engine::Engine;

use crate::message::SharedStateMessage;
use crate::physics::effect::Effect;
use crate::physics::utils::meters_between_scene_points;
use crate::{message::Message, physics::event::explosion::Explosion, NetworkMode};

impl Engine {
    pub fn tick_explosions(&self) -> Vec<Message> {
        let mut messages = vec![];
        let frame_i = self.local_state.get_frame_i();

        for explosion in self.local_state.explosions() {
            messages.extend(explosion.fx(self.local_state.get_frame_i()));

            if self.config.network_mode() == &NetworkMode::Server && explosion.effective(frame_i) {
                messages.extend(self.explosion_effects(explosion))
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

            // Simple for now, but if in vehicle, don't be affected
            if self.soldier_vehicle_place(soldier.uuid()).is_some() {
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

        for vehicle in self.shared_state.vehicles() {
            if vehicle.get_chassis_shape().contains(point) {
                messages.push(Message::SharedState(SharedStateMessage::PushPhysicsEffect(
                    Effect::VehicleShellImpact(*vehicle.uuid(), explosive_type.clone()),
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
