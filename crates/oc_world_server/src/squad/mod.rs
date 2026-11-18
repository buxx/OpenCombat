use derive_more::Constructor;
use oc_individual::{
    Individual,
    squad::{SquadIndex, SquadStatus, Update},
};
use oc_world::World;

use crate::{index::Indexes, runner};

pub mod update;

#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
    _index: &'a Indexes,
    i: SquadIndex,
}

impl<'a> Processor<'a> {
    pub fn step(self) -> Vec<runner::update::Update> {
        tracing::trace!(name="squad-step", i=?self.i);

        let squad = self.world.squad(self.i);
        let mut leader_index = squad.leader();

        // If leader can't lead anymore: next member able to lead take the lead
        let mut new_leader = None;
        if !self.world.individual(leader_index).can_leadership()
            && let Some(next) = squad
                .members
                .iter()
                .find(|i| self.world.individual(**i).can_leadership())
        {
            leader_index = *next;
            new_leader = Some(*next);
        }

        let leader = self.world.individual(leader_index);
        let members: Vec<&Individual> = squad
            .members
            .iter()
            .map(|i| self.world.individual(*i))
            .collect();

        let position = leader.position.into();
        let total = members.len();
        let dead = members.into_iter().filter(|m| m.status.is_dead()).count();
        let status = if dead == 0 {
            SquadStatus::Operational
        } else if dead == total {
            SquadStatus::Destroyed
        } else if dead * 2 >= total {
            SquadStatus::PartiallyDestroyed
        } else {
            SquadStatus::Casualities
        };
        let mut updates = vec![
            runner::update::Update::UpdateSquad(self.i, Update::SetPosition(position)),
            runner::update::Update::UpdateSquad(self.i, Update::SetActives((total - dead) as u8)),
            runner::update::Update::UpdateSquad(self.i, Update::SetStatus(status)),
        ];

        if let Some(new_leader) = new_leader {
            updates.push(runner::update::Update::UpdateSquad(
                self.i,
                Update::SetLeader(new_leader),
            ));
        }

        // Destroyed squad keeps no order
        if status == SquadStatus::Destroyed && !squad.orders.is_empty() {
            updates.push(runner::update::Update::UpdateSquad(
                self.i,
                Update::SetOrders(vec![]),
            ));
        }

        tracing::trace!(name = "squad-step-updates", i=?self.i, updates=?updates);
        updates
    }
}
