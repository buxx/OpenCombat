use crate::{engine::Engine, entity::soldier::Soldier, types::SquadUuid};

impl Engine {
    // TODO : choose soldier according to distance, weapon type, etc
    // TODO : choos soldier according to other squad targets (distribution)
    // TODO : don't return opponent id soldier is driver, working assistant, etc
    pub fn get_soldier_opponent(
        &self,
        soldier: &Soldier,
        squad_index: Option<&SquadUuid>,
    ) -> Option<&Soldier> {
        let mut visibles = self
            .local_state
            .visibilities()
            .visibles_soldiers_by_soldier(soldier);

        if let Some(squad_index) = squad_index {
            visibles.retain(|v| {
                self.shared_state
                    .soldier(v.to_soldier.expect("filtered previously"))
                    .squad_uuid()
                    == *squad_index
            })
        }

        // TODO : Use Millimeters as default unit instead Meters ?
        visibles.sort_by(|a, b| (a.distance.0 as u32).cmp(&(b.distance.0 as u32)));

        if let Some(visibility) = visibles.first() {
            return Some(
                self.shared_state.soldier(
                    visibility
                        .to_soldier
                        .expect("visibles_soldiers_by must return with to_soldier"),
                ),
            );
        }

        None
    }
}