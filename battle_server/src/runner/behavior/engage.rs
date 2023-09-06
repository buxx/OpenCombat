use battle_core::{
    entity::soldier::Soldier,
    game::cover::CoverFinder,
    order::Order,
    physics::path::{find_path, Direction, PathMode},
    types::{SoldierIndex, SquadUuid, WorldPath, WorldPaths},
};

use crate::runner::Runner;

impl Runner {
    pub fn propagate_engage_soldier(
        &self,
        squad_uuid: &SquadUuid,
        engaged_soldier_index: &SoldierIndex,
    ) -> Vec<(&Soldier, Order)> {
        let mut orders = vec![];
        let engaged_squad_index = self
            .battle_state
            .soldier(*engaged_soldier_index)
            .squad_uuid();
        let engaged_squad = self.battle_state.squad(engaged_squad_index);

        let mut after_grid_positions = vec![];
        'subordinates: for member in self
            .battle_state
            .squad(*squad_uuid)
            .subordinates()
            .iter()
            .map(|i| self.battle_state.soldier(**i))
        {
            let member_grid_point = self
                .battle_state
                .map()
                .grid_point_from_world_point(&member.world_point());
            if self
                .soldier_find_opponent_to_target(member, Some(&engaged_squad_index))
                .is_some()
            {
                log::debug!(
                    "Propagate engage soldier :: Member({}) :: have target",
                    member.uuid()
                );

                after_grid_positions.push(member_grid_point);
                orders.push((member, Order::EngageSquad(engaged_squad_index)));
            } else {
                // Subordinate can't targeted squad member. Try to find another place where he can
                let visible_targeted_squad_opponents: Vec<&Soldier> = engaged_squad
                    .members()
                    .iter()
                    .map(|i| self.battle_state.soldier(*i))
                    .filter(|s| {
                        self.battle_state
                            .soldier_is_visible_by_side(s, member.side())
                    })
                    .collect();

                for visible_opponent in &visible_targeted_squad_opponents {
                    if let Some(new_grid_point) = CoverFinder::new(&self.battle_state, &self.config)
                        .exclude_grid_points(after_grid_positions.clone())
                        .find_better_cover_point_from_point(
                            member,
                            &visible_opponent.world_point(),
                            true,
                        )
                    {
                        if let Some(grid_points_path) = find_path(
                            self.battle_state.map(),
                            &member_grid_point,
                            &new_grid_point,
                            true,
                            &PathMode::Walk,
                            &Some(Direction::from_angle(&member.get_looking_direction())),
                        ) {
                            after_grid_positions.push(new_grid_point);

                            let world_point_path = grid_points_path
                                .iter()
                                .map(|p| self.battle_state.map().world_point_from_grid_point(*p))
                                .collect();
                            let world_path = WorldPath::new(world_point_path);

                            log::debug!(
                                "Propagate engage soldier :: Member({}) :: no target :: Opponent({}) :: found new position ({}) :: found grid path ({:?})",
                                member.uuid(),
                                visible_opponent.uuid(),
                                new_grid_point,
                                grid_points_path,
                            );

                            orders.push((
                                member,
                                Order::MoveFastTo(WorldPaths::new(vec![world_path]), None),
                            ));
                            continue 'subordinates;
                        } else {
                            log::debug!(
                                "Propagate engage soldier :: Member({}) :: no target :: Opponent({}) :: found new position ({}) :: do not found grid path",
                                member.uuid(),
                                visible_opponent.uuid(),
                                new_grid_point,
                            );
                        };
                    } else {
                        log::debug!(
                            "Propagate engage soldier :: Member({}) :: no target :: Opponent({}) :: do not found new position",
                            member.uuid(),
                            visible_opponent.uuid(),
                        );
                    }
                }

                if visible_targeted_squad_opponents.is_empty() {
                    log::debug!(
                        "Propagate engage soldier :: Member({}) :: no target :: no visible enemy",
                        member.uuid(),
                    );
                }
            }
        }

        orders
    }
}
