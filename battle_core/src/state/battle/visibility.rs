use crate::{
    config::ServerConfig,
    entity::soldier::Soldier,
    game::Side,
    physics::visibility::Visibility,
    types::{SquadUuid, WorldPoint},
};

use super::BattleState;

impl BattleState {
    pub fn soldier_is_visible_by_side(&self, soldier: &Soldier, side: &Side) -> bool {
        for visibility in self.visibilities().visibles_soldiers().iter().filter(|v| {
            self.soldier(
                v.from_soldier
                    .expect("visibles_soldiers implies from_soldier"),
            )
            .side()
                == side
        }) {
            if visibility.to_soldier == Some(soldier.uuid()) {
                return true;
            }
        }

        false
    }

    pub fn soldier_squad_is_visible_by_side(&self, soldier: &Soldier, side: &Side) -> bool {
        for soldier_uuid in self.squad(soldier.squad_uuid()).members() {
            let squad_solider = self.soldier(*soldier_uuid);
            for visibility in self.visibilities().visibles_soldiers().iter().filter(|v| {
                self.soldier(
                    v.from_soldier
                        .expect("visibles_soldiers implies from_soldier"),
                )
                .side()
                    == side
            }) {
                if visibility.to_soldier == Some(squad_solider.uuid()) {
                    return true;
                }
            }
        }

        false
    }

    pub fn point_is_visible_by_squad(
        &self,
        config: &ServerConfig,
        point: &WorldPoint,
        squad_index: &SquadUuid,
        exclude_lasts: usize,
    ) -> bool {
        self.squad(*squad_index)
            .members()
            .iter()
            .map(|i| self.soldier(*i))
            .any(|s| {
                Visibility::between_soldier_and_point(config, s, point, self.map(), exclude_lasts)
                    .visible
            })
    }

    pub fn point_is_visible_by_soldier(
        &self,
        config: &ServerConfig,
        soldier: &Soldier,
        point: &WorldPoint,
        exclude_lasts: usize,
    ) -> Visibility {
        Visibility::between_soldier_and_point(config, soldier, point, self.map(), exclude_lasts)
    }
}
