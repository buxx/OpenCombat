use crate::{
    order::{marker::OrderMarker, Order},
    types::*,
};

use super::shared::SharedState;

impl SharedState {
    pub fn order_for(&self, entity_index: EntityIndex) -> Option<&Order> {
        let entity = self.entity(entity_index);
        self.given_orders().get(&entity.squad_uuid())
    }

    pub fn order_markers(&self) -> Vec<(OrderMarker, SquadUuid, WorldPoint, OrderMarkerIndex)> {
        for (squad_id, order) in self.all_orders() {
            let marker = order.marker();
            let squad = self.squad(squad_id);
            match order {
                Order::MoveTo(world_paths)
                | Order::MoveFastTo(world_paths)
                | Order::SneakTo(world_paths) => {
                    // Return one couple by move path (because can have multiple move paths))
                    return world_paths
                        .paths
                        .iter()
                        .enumerate()
                        .map(|(i, wp)| {
                            (
                                marker.clone(),
                                squad_id,
                                wp.last_point().expect("Must have point here"),
                                OrderMarkerIndex(i),
                            )
                        })
                        .collect();
                }
                Order::Defend(_) | Order::Hide(_) => {
                    let squad_leader = self.entity(squad.leader());
                    return vec![(
                        marker.clone(),
                        squad_id,
                        squad_leader.get_world_point(),
                        OrderMarkerIndex(0),
                    )];
                }
            }
        }

        vec![]
    }
}
