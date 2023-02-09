use battle_core::{
    entity::soldier::Soldier,
    order::Order,
    types::{SoldierIndex, SquadUuid},
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

        for member in self
            .battle_state
            .squad(*squad_uuid)
            .subordinates()
            .iter()
            .map(|i| self.battle_state.soldier(**i))
        {
            orders.push((member, Order::EngageSquad(engaged_squad_index)));
        }

        orders
    }
}
