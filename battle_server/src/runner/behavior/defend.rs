use battle_core::{
    behavior::Behavior,
    config::COVER_DISTANCE,
    entity::soldier::Soldier,
    map::find_cover_grid_point,
    order::Order,
    types::{GridPoint, SquadUuid, WorldPath, WorldPaths},
    utils::DebugPoint,
};

use crate::{
    game::squad::{squad_positions, Formation},
    runner::Runner,
};

impl Runner {
    pub fn propagate_defend(
        &self,
        squad_uuid: SquadUuid,
        _behavior: &Behavior,
    ) -> (Vec<(&Soldier, Order)>, Vec<DebugPoint>) {
        let squad = self.battle_state.squad(squad_uuid);
        let leader = self.battle_state.soldier(squad.leader());
        let mut orders = vec![];
        let debug_points = vec![];
        let mut already_used_cover_grid_points: Vec<GridPoint> = vec![];
        let map = self.battle_state.map();

        for (member_id, formation_position) in squad_positions(squad, Formation::Line, leader) {
            let soldier = self.battle_state.soldier(member_id);
            let grid_point = map.grid_point_from_world_point(&formation_position);
            if let Some((cover_grid_point, _debug_grid_points)) = find_cover_grid_point(
                &self.config,
                &grid_point,
                &map,
                &already_used_cover_grid_points,
                COVER_DISTANCE,
            ) {
                // FIXME BS NOW : enable it how by client ?
                // if self.local_state.debug_formation_positions {
                //     for debug_grid_point in debug_grid_points.iter() {
                //         debug_points.push(DebugPoint {
                //             frame_i: self.local_state.get_frame_i() + 120,
                //             point: self.world_point_from_grid_point(*debug_grid_point),
                //         })
                //     }
                // }

                let from_world_point = soldier.get_world_point();
                let cover_world_point = map.world_point_from_grid_point(cover_grid_point);
                already_used_cover_grid_points.push(cover_grid_point);

                let order = Order::MoveFastTo(WorldPaths::new(vec![WorldPath::new(vec![
                    from_world_point,
                    cover_world_point,
                ])]));
                orders.push((soldier, order));
            }
        }

        (orders, debug_points)
    }
}
