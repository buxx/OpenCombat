use std::collections::{HashMap, HashSet};

use crate::types::*;

use super::State;

impl State {
    pub fn update_squads(&mut self) {
        let mut new_squads = HashMap::new();

        for squad_uuid in self.unique_squad_ids() {
            let new_squad_leader = self
                .elect_squad_leader(squad_uuid)
                .expect("At this point, there must be at least one entity in the squad");
            let squad_entities = self.squad_entities(squad_uuid);
            new_squads.insert(
                squad_uuid,
                SquadComposition::new(new_squad_leader, squad_entities),
            );
        }

        self.squads = new_squads;
    }

    fn unique_squad_ids(&self) -> Vec<SquadUuid> {
        let mut all_squad_uuids: Vec<SquadUuid> =
            self.entities.iter().map(|e| e.squad_uuid()).collect();
        let unique_squad_uuids: HashSet<SquadUuid> = all_squad_uuids.drain(..).collect();
        unique_squad_uuids.into_iter().collect()
    }

    fn elect_squad_leader(&self, squad_uuid: SquadUuid) -> Option<EntityIndex> {
        let squad_entities = self.squad_entities(squad_uuid);

        if squad_entities.len() == 0 {
            return None;
        }

        // For now, election is done by get the first
        Some(
            *squad_entities
                .first()
                .expect("At this point, there must be at least one entity"),
        )
    }

    fn squad_entities(&self, squad_uuid: SquadUuid) -> Vec<EntityIndex> {
        self.entities
            .iter()
            .enumerate()
            .filter(|(_, e)| e.squad_uuid() == squad_uuid)
            .map(|(i, _)| EntityIndex(i))
            .collect()
    }
}
