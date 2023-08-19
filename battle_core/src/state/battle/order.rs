use crate::{
    game::Side,
    order::{marker::OrderMarker, Order},
    types::*,
};

use super::BattleState;

impl BattleState {
    // TODO : this func must clone things, this is not optimal
    // TODO : return type is too much complex
    pub fn order_markers(
        &self,
        side: &Side,
    ) -> Vec<(Order, OrderMarker, SquadUuid, WorldPoint, OrderMarkerIndex)> {
        let mut marker_data = vec![];

        for (squad_id, order) in self.all_orders(side) {
            let marker = order.marker();
            let squad = self.squad(squad_id);
            match &order {
                Order::MoveTo(world_paths, _)
                | Order::MoveFastTo(world_paths, _)
                | Order::SneakTo(world_paths, _) => {
                    // Return one couple by move path (because can have multiple move paths))
                    marker_data.extend::<Vec<(
                        Order,
                        OrderMarker,
                        SquadUuid,
                        WorldPoint,
                        OrderMarkerIndex,
                    )>>(
                        world_paths
                            .paths
                            .iter()
                            .enumerate()
                            .map(|(i, wp)| {
                                (
                                    order.clone(),
                                    marker.clone().unwrap(),  // FIXME : unwrap to delete
                                    squad_id,
                                    wp.last_point().expect("Must have point here"),
                                    OrderMarkerIndex(i),
                                )
                            })
                            .collect(),
                    );
                }
                Order::Defend(_) | Order::Hide(_) => {
                    let squad_leader = self.soldier(squad.leader());
                    marker_data.push((
                        order.clone(),
                        marker.clone().unwrap(), // FIXME unwrap to remove
                        squad_id,
                        squad_leader.world_point(),
                        OrderMarkerIndex(0),
                    ));
                }
                Order::Idle => {}
                Order::EngageSquad(squad_index) => {
                    let squad = self.squad(*squad_index);
                    let leader = self.soldier(squad.leader());
                    marker_data.push((
                        order.clone(),
                        marker.clone().unwrap(), // FIXME unwrap to remove
                        squad_id,
                        leader.world_point(),
                        OrderMarkerIndex(0),
                    ));
                }
                Order::SuppressFire(point) => {
                    marker_data.push((
                        order.clone(),
                        marker.clone().unwrap(), // FIXME unwrap to remove
                        squad_id,
                        *point,
                        OrderMarkerIndex(0),
                    ));
                }
            }
        }

        marker_data
    }
}
