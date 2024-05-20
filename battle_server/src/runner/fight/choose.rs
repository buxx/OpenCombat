use rand::seq::SliceRandom;

use battle_core::{
    entity::soldier::Soldier,
    physics::{utils::distance_between_points, visibility::Visibility},
    state::battle::BattleState,
    types::{Distance, SoldierIndex, SquadUuid},
};

use crate::runner::Runner;

pub const NEAR_SOLDIERS_DISTANCE_METERS: i64 = 7;

pub enum ChooseMethod {
    RandomFromNearest,
}
impl ChooseMethod {
    fn choose(&self, battle_state: &BattleState, soldiers: Vec<&Soldier>) -> Option<SoldierIndex> {
        match self {
            Self::RandomFromNearest => self.choose_random_from_nearest(battle_state, soldiers),
        }
    }

    fn choose_random_from_nearest(
        &self,
        _battle_state: &BattleState,
        soldiers: Vec<&Soldier>,
    ) -> Option<SoldierIndex> {
        if let Some(soldier) = soldiers.first() {
            let soldier_position = soldier.world_point();
            let near_soldiers: Vec<&Soldier> = soldiers
                .into_iter()
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

impl Runner {
    // TODO : choose soldier according to distance, weapon type, etc
    // TODO : choose soldier according to other squad targets (distribution)
    // TODO : don't make it if soldier is driver, working assistant, etc
    pub fn soldier_find_opponent_to_target(
        &self,
        soldier: &Soldier,
        squad_index: Option<&SquadUuid>,
        choose_method: &ChooseMethod,
    ) -> Option<&Soldier> {
        let around_soldiers: Vec<SoldierIndex> = self
            .battle_state
            .get_circle_side_soldiers_able_to_see(
                soldier.side(),
                &soldier.world_point(),
                &Distance::from_meters(20),
            )
            .iter()
            .map(|s| s.uuid())
            .collect();
        let mut visibles: Vec<&Soldier> = self
            .battle_state
            .visibilities()
            // FIXME BS NOW: !!! visible by near soldiers instead of all side
            .visibles_soldiers_by_soldiers(around_soldiers)
            .iter()
            .map(|s| self.battle_state.soldier(*s))
            .collect();

        visibles.retain(|s| s.can_be_designed_as_target());

        if let Some(squad_index) = squad_index {
            visibles.retain(|s| s.squad_uuid() == *squad_index)
        }

        // Why this sort ?
        // visibles.sort_by(|a, b| {
        //     a.distance
        //         .millimeters()
        //         .partial_cmp(&b.distance.millimeters())
        //         .expect("Must be i64")
        // });

        if soldier.behavior().is_hide() {
            visibles.retain(|s| {
                distance_between_points(&soldier.world_point(), &s.world_point())
                    <= self.config.hide_maximum_rayon
            })
        }

        choose_method
            .choose(&self.battle_state, visibles)
            .map(|i| self.battle_state.soldier(i))
    }
}
