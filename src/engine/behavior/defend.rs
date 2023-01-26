use crate::{
    behavior::Behavior,
    engine::Engine,
    entity::soldier::Soldier,
    game::squad::{squad_positions, Formation},
    map::find_cover_grid_point,
    order::Order,
    types::{GridPoint, SquadUuid, WorldPath, WorldPaths},
    utils::DebugPoint,
};

impl Engine {
    pub fn propagate_defend(
        &self,
        squad_uuid: SquadUuid,
        _behavior: &Behavior,
    ) -> (Vec<(&Soldier, Order)>, Vec<DebugPoint>) {
        let squad = self.shared_state.squad(squad_uuid);
        let mut orders = vec![];
        let mut debug_points = vec![];
        let mut already_used_cover_grid_points: Vec<GridPoint> = vec![];

        for (member_id, formation_position) in
            squad_positions(squad, Formation::Line, &self.shared_state)
        {
            let soldier = self.shared_state.soldier(member_id);
            let grid_point = self.grid_point_from_world_point(formation_position);
            if let Some((cover_grid_point, debug_grid_points)) =
                find_cover_grid_point(&grid_point, &self.map, &already_used_cover_grid_points)
            {
                if self.local_state.get_debug_level().formation_positions() {
                    for debug_grid_point in debug_grid_points.iter() {
                        debug_points.push(DebugPoint {
                            frame_i: self.local_state.get_frame_i() + 120,
                            point: self.world_point_from_grid_point(*debug_grid_point),
                        })
                    }
                }

                let from_world_point = soldier.get_world_point();
                let cover_world_point = self.world_point_from_grid_point(cover_grid_point);
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
