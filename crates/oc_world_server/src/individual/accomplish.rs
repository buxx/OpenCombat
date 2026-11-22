use oc_individual::{Update, behavior::Intent, order::Order};
use oc_utils::d2::{AlmostEqual, Direction};

use super::{PATH_STEP_TOLERANCE, POSITION_TOLERANCE, Processor, situation::Situation};
use crate::runner;

impl Processor<'_> {
    /// Builds the "accomplished" update batch for this individual, appending squad-level updates
    /// when this individual holds the squad order (squad leader, or vehicle crew member when
    /// leader is in a vehicle, see [`Processor::squad_order_holder`]).
    pub fn accomplished_updates(&self, direction: Direction) -> Vec<runner::update::Update> {
        let squad_i = self.index.individual_squad(self.i);
        let squad = self.world.squad(squad_i);
        let individual = self.world.individual(self.i);
        let leader_embarked = self.world.individual(squad.leader()).embarked.is_some();
        // In vehicle, crew can receive the squad order from leader again (distributed before it
        // was accomplished): only the current squad order can be accomplished.
        let is_squad_leader = self.squad_order_holder(squad) == Some(self.i)
            && (!leader_embarked || individual.orders.first() == squad.orders.first());

        let i_accomplish = Update::Accomplished;
        let i_accomplish = runner::update::Update::UpdateIndividual(self.i, i_accomplish);

        let i_idle = Update::SetIntent(Intent::Idle(direction));
        let i_idle = runner::update::Update::UpdateIndividual(self.i, i_idle);

        let mut updates = vec![i_accomplish, i_idle];

        if is_squad_leader && !self.members_still_throwing(squad) {
            // FIXME: Must wait all members finished associated order (only relevant for MoveTo,
            // kept here since Idle currently shares the same behavior).
            let accomplish = oc_individual::squad::Update::Accomplished;
            let accomplish = runner::update::Update::UpdateSquad(squad_i, accomplish);

            let orders = squad.orders.clone().into_iter().skip(1).collect();
            let orders = oc_individual::squad::Update::SetOrders(orders);
            let orders = runner::update::Update::UpdateSquad(squad_i, orders);

            updates.extend(vec![accomplish, orders]);
        }

        updates
    }

    /// Grenade and smoke orders are finished for the squad when all members threw: leader waits
    /// for members which still hold the same order and own a matching grenade.
    fn members_still_throwing(&self, squad: &oc_individual::squad::Squad) -> bool {
        let Some(order) = self.world.individual(self.i).orders.first() else {
            return false;
        };
        if !matches!(order, Order::Grenade(_) | Order::Smoke(_)) {
            return false;
        }

        squad
            .members
            .iter()
            .filter(|member| **member != self.i)
            .map(|member| self.world.individual(*member))
            .any(|member| {
                // Members may throw aside of the leader target (smoke screen): compare order kinds
                member.status.is_target()
                    && member
                        .orders
                        .iter()
                        .any(|o| std::mem::discriminant(o) == std::mem::discriminant(order))
                    && self.matching_grenade(member, order).is_some()
            })
    }

    /// Determine if current order (or order step) is accomplished
    pub fn accomplished(&self, _situation: &Situation) -> Option<Vec<runner::update::Update>> {
        let individual = self.world.individual(self.i);
        let Some(order) = individual.orders.first() else {
            tracing::trace!(name = "individual-step-accomplished-no-order", i=?self.i);
            return None;
        };
        let mut updates = None;
        let direction = individual.gesture.direction();

        match order {
            Order::Idle => {
                tracing::trace!(name = "individual-step-accomplished-idle-finished", i = ?self.i);
                updates = Some(self.accomplished_updates(direction));
            }
            Order::MoveTo(position) | Order::MoveFastTo(position) | Order::SneakTo(position) => {
                // Vehicle stops when near its destination (see `oc_vehicle::Vehicle::want_stop`):
                // its crew must consider the move accomplished with the same tolerance.
                let tolerance = match individual.embarked {
                    Some(_) => oc_vehicle::DESTINATION_TOLERANCE,
                    None => POSITION_TOLERANCE,
                };
                if position.almost_equal(individual.position, tolerance) {
                    tracing::trace!(name = "individual-step-accomplished-move-to-finished", i = ?self.i);
                    updates = Some(self.accomplished_updates(direction));
                }
            }
            // Defend and hide never finish
            Order::Defend(_) | Order::Hide(_) => {}
            // FIXME: must finish when ammo low
            Order::Suppress(_) => {}
            // Grenade and smoke orders are skipped when individual has no (more) matching grenade.
            Order::Grenade(_) | Order::Smoke(_) => {
                if self.matching_grenade(individual, order).is_none() {
                    tracing::trace!(name = "individual-step-accomplished-no-grenade", i = ?self.i);
                    updates = Some(self.accomplished_updates(direction));
                }
            }
            // Engage finishes when target is no longer operational. Losing sight of it only does
            // not: it can be seen again, or still be seen by another individual of the squad.
            Order::Engage(individual) => {
                if !self.world.individual(*individual).status.is_target() {
                    tracing::trace!(name = "individual-step-accomplished-engage-finished", i=?self.i, i2=?individual);
                    updates = Some(self.accomplished_updates(direction));
                }
            }
        };

        if updates.is_some() {
            return updates;
        }

        // Test move intents to trigger "next step" when step reached
        match &individual.intent {
            // Idle/Defend/Hide/Engage/Suppress never finish
            Intent::Idle(_)
            | Intent::Defend(_)
            | Intent::Hide(_)
            | Intent::Engage(_)
            | Intent::Suppress(_)
            | Intent::Grenade(_)
            | Intent::Smoke(_) => {}
            Intent::MoveTo(_, move_path)
            | Intent::MoveFastTo(_, move_path)
            | Intent::SneakTo(_, move_path) => {
                let Some(next) = move_path.iter().next() else {
                    tracing::trace!(name = "individual-step-accomplished-intent-move-to-no-next", i=?self.i);
                    return updates;
                };

                if next.almost_equal(individual.position, PATH_STEP_TOLERANCE) {
                    let update = Update::MoveStepAccomplished;
                    let update = runner::update::Update::UpdateIndividual(self.i, update);
                    updates.get_or_insert(vec![]).push(update);
                }
            }
        }

        updates
    }
}
