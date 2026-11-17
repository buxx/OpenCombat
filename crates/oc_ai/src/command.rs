use oc_individual::{order::Order, squad::SquadIndex};
use oc_root::geo::WorldVec2;
use oc_utils::d2::Angle;
use oc_world::support::SupportType;

use crate::perception::SquadView;

/// Action decided by an AI. Mirror of what a human player can send to the server.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Replace orders of given squad (must be a squad of AI side)
    SetOrders(SquadIndex, Vec<Order>),
    /// Move squad leader during deployment phase, members being placed in a line along given
    /// angle around it
    SetLeaderPosition(SquadIndex, WorldVec2, Angle),
    /// Request a support (artillery, etc.) at given position
    RequestSupport(SupportType, WorldVec2),
}

impl Command {
    /// Replace squad orders by `orders`. None if its current order already is the first one
    /// (orders are only given when they change, to let squads accomplish them).
    pub fn set_orders_if_changed(squad: &SquadView, orders: Vec<Order>) -> Option<Command> {
        (squad.orders.first() != orders.first()).then_some(Command::SetOrders(squad.index, orders))
    }
}
