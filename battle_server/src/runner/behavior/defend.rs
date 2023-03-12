use battle_core::{
    behavior::Behavior,
    entity::soldier::Soldier,
    game::cover::find_cover_points,
    order::Order,
    types::{SquadUuid, WorldPath, WorldPaths},
    utils::NewDebugPoint,
};

use crate::runner::Runner;

impl Runner {
    pub fn propagate_defend(
        &self,
        squad_uuid: SquadUuid,
        _behavior: &Behavior,
    ) -> (Vec<(&Soldier, Order)>, Vec<NewDebugPoint>) {
        let squad = self.battle_state.squad(squad_uuid);
        let leader = self.battle_state.soldier(squad.leader());
        let mut orders = vec![];

        let (moves, debug_points) =
            find_cover_points(squad, leader, &self.battle_state, &self.config, None);

        for (member_id, from_world_point, cover_world_point) in moves {
            let order = Order::MoveFastTo(WorldPaths::new(vec![WorldPath::new(vec![
                from_world_point,
                cover_world_point,
            ])]));
            orders.push((self.battle_state.soldier(member_id), order));
        }

        (orders, debug_points)
    }
}
