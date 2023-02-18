use battle_core::audio::Sound;
use battle_core::entity::soldier::Soldier;
use battle_core::entity::vehicle::Vehicle;
use battle_core::game::explosive::ExplosiveType;
use battle_core::physics::event::explosion::Explosion;
use battle_core::physics::utils::meters_between_world_points;
use battle_core::state::client::ClientStateMessage;

use battle_core::types::Meters;
use rand::seq::SliceRandom;

use crate::runner::message::RunnerMessage;
use crate::runner::Runner;

impl Runner {
    pub fn tick_explosions(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_explosions");
        let mut messages = vec![];

        for explosion in self.battle_state.explosions() {
            if explosion.effective(self.frame_i) {
                messages.extend(self.explosion_effects(explosion))
            }
        }

        messages
    }

    // TODO : find algorithm kill/injure about explosives + terrain + position
    fn explosion_effects(&self, explosion: &Explosion) -> Vec<RunnerMessage> {
        puffin::profile_scope!(
            "explosion_effects",
            format!("start={} end={}", explosion.start(), explosion.end())
        );
        let mut messages = vec![];
        let point = explosion.point();
        let explosive_type = explosion.type_();
        let _blast = explosive_type.blast();

        for soldier in self.battle_state.soldiers() {
            if !soldier.can_feel_explosion() {
                continue;
            }

            // Simple for now, but if in vehicle, don't be affected
            if self
                .battle_state
                .soldier_vehicle_place(soldier.uuid())
                .is_some()
            {
                continue;
            }

            let distance = meters_between_world_points(&soldier.get_world_point(), point);

            if distance.0 < 4.0 {
                messages.extend(self.killing_blast_effects(soldier));
            } else if distance.0 < 7.0 {
                messages.extend(self.stunning_blast_effects(soldier));
            } else if distance.0 < 75.0 {
                messages.extend(self.proximity_blast_effects(soldier, distance));
            }
        }

        for vehicle in self.battle_state.vehicles() {
            if vehicle.get_chassis_shape().contains(point) {
                messages.extend(self.vehicle_shell_impact_effects(vehicle, explosive_type));
            }
        }

        messages
    }

    fn killing_blast_effects(&self, soldier: &Soldier) -> Vec<RunnerMessage> {
        puffin::profile_scope!("killing_blast_effects", soldier.uuid().to_string());
        let mut messages = self.soldier_die(soldier.uuid());

        if soldier.can_produce_sound() {
            let pick_from = vec![
                Sound::MaleScreaming1,
                Sound::MaleScreaming2,
                Sound::MaleScreaming3,
                Sound::MaleScreaming4,
                Sound::MaleScreaming5,
                Sound::MaleDie1,
                Sound::MaleDie2,
                Sound::MaleDie3,
                Sound::MaleDie4,
                Sound::MaleDie5,
                Sound::MaleDie6,
                Sound::MaleDie7,
                Sound::MaleDie8,
            ];
            messages.push(RunnerMessage::ClientsState(
                ClientStateMessage::PlayBattleSound(
                    *pick_from
                        .choose(&mut rand::thread_rng())
                        .expect("Must one be chosen"),
                ),
            ))
        }

        messages
    }

    fn stunning_blast_effects(&self, soldier: &Soldier) -> Vec<RunnerMessage> {
        puffin::profile_scope!("stunning_blast_effects", soldier.uuid().to_string());
        self.soldier_blast_stunned(soldier.uuid())
    }

    fn proximity_blast_effects(&self, soldier: &Soldier, distance: Meters) -> Vec<RunnerMessage> {
        puffin::profile_scope!("proximity_blast_effects", soldier.uuid().to_string());
        self.soldier_blast(soldier.uuid(), distance)
    }

    fn vehicle_shell_impact_effects(
        &self,
        vehicle: &Vehicle,
        _explosive: &ExplosiveType,
    ) -> Vec<RunnerMessage> {
        puffin::profile_scope!("vehicle_shell_impact_effects", vehicle.uuid().to_string());
        // TODO effects on soldiers (with a real explosive algorithm)
        let pick_from = vec![Sound::MetalHit1];
        return vec![RunnerMessage::ClientsState(
            ClientStateMessage::PlayBattleSound(
                *pick_from
                    .choose(&mut rand::thread_rng())
                    .expect("Must one be chosen"),
            ),
        )];
    }
}
