use battle_core::{
    behavior::Behavior,
    entity::{soldier::Soldier, vehicle::OnBoardPlace},
    game::squad::{squad_positions, Formation},
    order::Order,
    physics::path::{find_path, PathMode},
    types::{SquadUuid, WorldPath, WorldPaths},
};

use crate::runner::Runner;

impl Runner {
    pub fn propagate_move(
        &self,
        squad_uuid: SquadUuid,
        behavior: &Behavior,
    ) -> Vec<(&Soldier, Order)> {
        let mut behaviors = vec![];
        let squad = self.battle_state.squad(squad_uuid);
        let leader = self.battle_state.soldier(squad.leader());

        for (soldier_index, point) in squad_positions(squad, Formation::Line, leader, None) {
            let soldier = self.battle_state.soldier(soldier_index);
            let map = self.battle_state.map();
            if let Some(grid_path) = find_path(
                map,
                &map.grid_point_from_world_point(&soldier.get_world_point()),
                &map.grid_point_from_world_point(&point),
                true,
                &PathMode::Walk,
                &None,
            ) {
                let world_path = grid_path
                    .iter()
                    .map(|p| map.world_point_from_grid_point(*p))
                    .collect();
                let world_paths = WorldPaths::new(vec![WorldPath::new(world_path)]);

                let order = match behavior {
                    Behavior::MoveTo(_) => Order::MoveTo(world_paths),
                    Behavior::MoveFastTo(_) => Order::MoveFastTo(world_paths),
                    Behavior::SneakTo(_) => Order::SneakTo(world_paths),
                    _ => unreachable!(),
                };
                behaviors.push((soldier, order))
            }
        }

        behaviors
    }

    pub fn propagate_drive(
        &self,
        squad_uuid: SquadUuid,
        behavior: &Behavior,
    ) -> Vec<(&Soldier, Order)> {
        let squad = self.battle_state.squad(squad_uuid);

        for member_index in squad.members() {
            if let Some((_, place)) = self.battle_state.soldier_board(*member_index) {
                if place == &OnBoardPlace::Driver {
                    let soldier = self.battle_state.soldier(*member_index);
                    let paths = match &behavior {
                        Behavior::MoveTo(paths)
                        | Behavior::MoveFastTo(paths)
                        | Behavior::SneakTo(paths) => paths,
                        _ => {
                            unreachable!()
                        }
                    };
                    return vec![(soldier, Order::MoveTo(paths.clone()))];
                }
            }
        }

        vec![]
    }
}
