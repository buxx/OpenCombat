use battle_core::{entity::soldier::Soldier, types::SquadUuid};

use crate::runner::Runner;

impl Runner {
    // TODO : choose soldier according to distance, weapon type, etc
    // TODO : choose soldier according to other squad targets (distribution)
    // TODO : don't make it if soldier is driver, working assistant, etc
    pub fn soldier_find_opponent_to_target(
        &self,
        soldier: &Soldier,
        squad_index: Option<&SquadUuid>,
    ) -> Option<&Soldier> {
        let mut visibles = self
            .battle_state
            .visibilities()
            .visibles_soldiers_by_soldier(soldier);

        visibles.retain(|v| {
            self.battle_state
                .soldier(v.to_soldier.expect("filtered previously"))
                .can_be_designed_as_target()
        });

        if let Some(squad_index) = squad_index {
            visibles.retain(|v| {
                self.battle_state
                    .soldier(v.to_soldier.expect("filtered previously"))
                    .squad_uuid()
                    == *squad_index
            })
        }

        visibles.sort_by(|a, b| {
            a.distance
                .millimeters()
                .partial_cmp(&b.distance.millimeters())
                .expect("Must be i64")
        });

        if soldier.behavior().is_hide() {
            visibles.retain(|v| v.distance <= self.config.hide_maximum_rayon)
        }

        if let Some(visibility) = visibles.first() {
            return Some(
                self.battle_state.soldier(
                    visibility
                        .to_soldier
                        .expect("visibles_soldiers_by must return with to_soldier"),
                ),
            );
        }

        None
    }
}
