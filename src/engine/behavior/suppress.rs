use crate::{
    engine::Engine,
    entity::soldier::Soldier,
    order::Order,
    types::{SquadUuid, WorldPoint},
};

impl Engine {
    pub fn propagate_suppress_fire(
        &self,
        squad_uuid: SquadUuid,
        point: &WorldPoint,
    ) -> Vec<(&Soldier, Order)> {
        let mut orders = vec![];

        for member in self
            .shared_state
            .squad(squad_uuid)
            .subordinates()
            .iter()
            .map(|i| self.shared_state.soldier(**i))
        {
            orders.push((member, Order::SuppressFire(*point)));
        }

        orders
    }
}
