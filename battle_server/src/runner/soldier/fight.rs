use super::SoldierRunner;
use rand::seq::SliceRandom;

use battle_core::{
    entity::soldier::Soldier,
    physics::{utils::distance_between_points, visibility::Visibility},
    state::battle::BattleState,
    types::{Distance, SoldierIndex, SquadUuid},
};

pub const NEAR_SOLDIERS_DISTANCE_METERS: i64 = 7;

pub enum ChooseMethod {
    RandomFromNearest,
}
impl ChooseMethod {
    fn choose(
        &self,
        battle_state: &BattleState,
        visibles: Vec<&Visibility>,
    ) -> Option<SoldierIndex> {
        match self {
            Self::RandomFromNearest => self.choose_random_from_nearest(battle_state, visibles),
        }
    }

    fn choose_random_from_nearest(
        &self,
        battle_state: &BattleState,
        visibles: Vec<&Visibility>,
    ) -> Option<SoldierIndex> {
        if let Some(visibility) = visibles.first() {
            let soldier = battle_state.soldier(
                visibility
                    .to_soldier
                    .expect("visibles_soldiers_by must returned with to_soldier"),
            );
            let soldier_position = soldier.world_point();
            let near_soldiers: Vec<&Soldier> = visibles
                .iter()
                .map(|v| {
                    battle_state.soldier(
                        v.to_soldier
                            .expect("visibles_soldiers_by must returned with to_soldier"),
                    )
                })
                .filter(|s| {
                    distance_between_points(&soldier_position, &s.world_point())
                        < Distance::from_meters(NEAR_SOLDIERS_DISTANCE_METERS)
                })
                .collect();

            return near_soldiers
                .choose(&mut rand::thread_rng())
                .map(|s| s.uuid());
        }

        None
    }
}

impl SoldierRunner {
    // TODO : choose soldier according to distance, weapon type, etc
    // TODO : choose soldier according to other squad targets (distribution)
    // TODO : don't make it if soldier is driver, working assistant, etc
    pub fn soldier_find_opponent_to_target(
        &self,
        soldier: &Soldier,
        squad_index: Option<&SquadUuid>,
        method: &ChooseMethod,
    ) -> Option<&Soldier> {
        let mut visibles = self
            .battle_state()
            .visibilities()
            .visibles_soldiers_by_soldier(soldier);

        visibles.retain(|v| {
            self.battle_state()
                .soldier(v.to_soldier.expect("filtered previously"))
                .can_be_designed_as_target()
        });

        if let Some(squad_index) = squad_index {
            visibles.retain(|v| {
                self.battle_state()
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

        method
            .choose(&self.battle_state(), visibles)
            .map(|i| self.battle_state().soldier(i))
    }
}
