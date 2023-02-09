use battle_core::{
    entity::soldier::Soldier,
    order::Order,
    types::{SquadUuid, WorldPoint},
};

use crate::runner::Runner;

impl Runner {
    pub fn propagate_suppress_fire(
        &self,
        squad_uuid: SquadUuid,
        point: &WorldPoint,
    ) -> Vec<(&Soldier, Order)> {
        let mut orders = vec![];

        for member in self
            .battle_state
            .squad(squad_uuid)
            .subordinates()
            .iter()
            .map(|i| self.battle_state.soldier(**i))
        {
            orders.push((member, Order::SuppressFire(*point)));
        }

        orders
    }
}
