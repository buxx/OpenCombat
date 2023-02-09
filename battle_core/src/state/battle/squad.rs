use std::collections::{HashMap, HashSet};

use crate::{
    behavior::BehaviorMode,
    types::{SoldierIndex, SquadComposition, SquadUuid},
};

use super::BattleState;

impl BattleState {
    pub fn update_squads(&mut self) {
        let mut new_squads = HashMap::new();

        for squad_uuid in self.unique_squad_ids() {
            let new_squad_leader = self
                .elect_squad_leader(squad_uuid)
                .expect("At this point, there must be at least one soldier in the squad");
            let squad_entities = self.squad_entities(squad_uuid);
            new_squads.insert(
                squad_uuid,
                SquadComposition::new(new_squad_leader, squad_entities),
            );
        }

        self.set_squads(new_squads);
    }

    fn unique_squad_ids(&self) -> Vec<SquadUuid> {
        let mut all_squad_uuids: Vec<SquadUuid> =
            self.soldiers().iter().map(|e| e.squad_uuid()).collect();
        let unique_squad_uuids: HashSet<SquadUuid> = all_squad_uuids.drain(..).collect();
        unique_squad_uuids.into_iter().collect()
    }

    fn elect_squad_leader(&self, squad_uuid: SquadUuid) -> Option<SoldierIndex> {
        let squad_entities = self.squad_entities(squad_uuid);

        if squad_entities.len() == 0 {
            return None;
        }

        // For now, election is done by get the first
        Some(
            *squad_entities
                .first()
                .expect("At this point, there must be at least one soldier"),
        )
    }

    fn squad_entities(&self, squad_uuid: SquadUuid) -> Vec<SoldierIndex> {
        self.soldiers()
            .iter()
            .enumerate()
            .filter(|(_, e)| e.squad_uuid() == squad_uuid)
            .map(|(i, _)| SoldierIndex(i))
            .collect()
    }

    pub fn squad_behavior_mode(&self, squad_index: &SquadUuid) -> BehaviorMode {
        let squad = self.squad(*squad_index);
        self.soldier_behavior_mode(self.soldier(squad.leader()))
    }
}
