use battle_core::{
    behavior::Behavior,
    entity::{soldier::Soldier, vehicle::OnBoardPlace},
    game::cover::CoverFinder,
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

        let (moves, debug_points) = CoverFinder::new(&self.battle_state, &self.config)
            .find_arbitrary_cover_points(squad, leader);

        for (member_id, from_world_point, cover_world_point) in moves {
            let order = Order::MoveFastTo(WorldPaths::new(vec![WorldPath::new(vec![
                from_world_point,
                cover_world_point,
            ])]));
            orders.push((self.battle_state.soldier(member_id), order));
        }

        (orders, debug_points)
    }

    pub fn propagate_rotate(
        &self,
        squad_uuid: SquadUuid,
        behavior: &Behavior,
    ) -> (Vec<(&Soldier, Order)>, Vec<NewDebugPoint>) {
        let squad = self.battle_state.squad(squad_uuid);

        for member_index in squad.members() {
            if let Some((_, place)) = self.battle_state.soldier_board(*member_index) {
                if place == &OnBoardPlace::Driver {
                    let soldier = self.battle_state.soldier(*member_index);
                    let order = match &behavior {
                        Behavior::Defend(angle) => Order::Defend(*angle),
                        Behavior::Hide(angle) => Order::Hide(*angle),
                        _ => {
                            unreachable!()
                        }
                    };
                    return (vec![(soldier, order)], vec![]);
                }
            }
        }

        (vec![], vec![])
    }
}
