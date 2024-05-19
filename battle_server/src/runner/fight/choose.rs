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
        method: &ChooseMethod,
    ) -> Option<&Soldier> {
        // FIXME BS NOW bug: il faut choisir parmis les soldats visibles de son SIDE
        // PUIS, quand test si soldat point touchable (appel a soldier_able_to_fire_on_point)
        // Il faut donner l'objet soldier target (ou simplement tester si soldat target est visible && pas d'obstacle entre ? exclude quelques tuiles a la fin ?)
        // et pas juste le point (car on doit pouvoir cibler un soldat connu mais pas vu direct)
        let mut visibles: Vec<&Soldier> = self
            .battle_state
            .visibilities()
            .visibles_soldiers_by_side(soldier.side())
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

        method
            .choose(&self.battle_state, visibles)
            .map(|i| self.battle_state.soldier(i))
    }
}
