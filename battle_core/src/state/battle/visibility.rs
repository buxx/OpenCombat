use crate::{
    config::TerrainTileOpacity,
    entity::soldier::Soldier,
    game::Side,
    physics::visibility::Visibility,
    types::{SquadUuid, WorldPoint},
};

use super::BattleState;

impl BattleState {
    pub fn soldier_is_visible_by_side(&self, soldier: &Soldier, side: &Side) -> bool {
        for visibility in self
            .visibilities()
            .visibles_soldiers()
            .iter()
            .filter(|v| self.soldier(v.from_soldier).get_side() == side)
        {
            if visibility.to_soldier == Some(soldier.uuid()) {
                return true;
            }
        }

        false
    }

    pub fn point_is_visible_by_squad(
        &self,
        config: &impl TerrainTileOpacity,
        point: &WorldPoint,
        squad_index: &SquadUuid,
    ) -> bool {
        self.squad(*squad_index)
            .members()
            .iter()
            .map(|i| self.soldier(*i))
            .any(|s| Visibility::between_soldier_and_point(config, s, point, &self.map()).visible)
    }

    pub fn point_is_visible_by_soldier(
        &self,
        config: &impl TerrainTileOpacity,
        soldier: &Soldier,
        point: &WorldPoint,
    ) -> bool {
        Visibility::between_soldier_and_point(config, soldier, point, self.map()).visible
    }
}
