use crate::runner::update::Update;
use derive_more::Constructor;
use oc_individual::Individual;
use oc_root::{
    battle::{BattlePhase, End},
    side::Side,
};
use oc_utils::polygon::Polygon;
use oc_world::{
    World,
    flag::{Flag, Flags, Ownership},
    morale::Morale,
};

#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
}

impl Processor<'_> {
    pub fn compute(&self) -> Vec<Update> {
        let mut updates = vec![
            Update::UpdateFlags(self.compute_flags()),
            Update::UpdateMorale(self.compute_morale()),
        ];

        updates.extend(self.compute_support());

        let (time, ended) = self.time();
        updates.extend(time);

        if !ended {
            updates.extend(self.morale_end());
        }

        updates
    }

    fn compute_flags(&self) -> Flags {
        let mut flags = self.world.flags.clone();

        for (flag, ownership) in flags.iter_mut() {
            if let Some(new) = self.flag_ownership(flag) {
                *ownership = new;
            }
        }

        flags
    }

    fn flag_ownership(&self, flag: &Flag) -> Option<Ownership> {
        let mut side_a = false;
        let mut side_b = false;

        for individual in self.operational_individuals_in(flag) {
            match individual.side {
                Side::A => side_a = true,
                Side::B => side_b = true,
            }
            if side_a && side_b {
                break;
            }
        }

        match (side_a, side_b) {
            (true, true) => Some(Ownership::Both),
            (true, false) => Some(Ownership::A),
            (false, true) => Some(Ownership::B),
            (false, false) => None,
        }
    }

    fn operational_individuals_in(&self, flag: &Flag) -> impl Iterator<Item = &Individual> {
        self.world
            .individuals()
            .iter()
            .filter(|individual| Self::is_operational(individual))
            .filter(|individual| flag.contains(individual.position.into()))
    }

    fn is_operational(individual: &Individual) -> bool {
        matches!(individual.status, oc_individual::Status::Operational)
    }

    fn compute_morale(&self) -> Morale {
        Morale {
            a: self.side_morale(Side::A),
            b: self.side_morale(Side::B),
        }
    }

    fn side_morale(&self, side: Side) -> f32 {
        let individuals = self
            .world
            .individuals()
            .iter()
            .filter(|individual| individual.side == side);
        let total = individuals.clone().count();
        let operational = individuals
            .filter(|individual| Self::is_operational(individual))
            .count();

        match total {
            0 => 0.,
            _ => operational as f32 / total as f32 * 100.,
        }
    }

    fn compute_support(&self) -> Vec<Update> {
        if self.world.phase != BattlePhase::Fight {
            return vec![];
        }

        let mut updates = vec![];

        for pending in &self.world.pending_supports {
            if !pending.is_started() {
                tracing::trace!(name="world-start-support", pending=?pending);
                updates.push(Update::StartPendingSupport(pending.clone()));
            }

            let Some(support) = self.world.mod_.support(pending.type_.name()) else {
                tracing::warn!(name="world-support-unknown", pending=?pending);
                updates.push(Update::RemovePendingSupport(pending.clone()));
                continue;
            };

            if !pending.is_ready(support, self.world.w.speed()) {
                tracing::trace!(name="world-support-not-ready", pending=?pending);
                continue;
            }

            tracing::debug!(name="world-support-ready", pending=?pending);
            updates.push(Update::RealizeSupport(pending.clone()));
            updates.push(Update::RemovePendingSupport(pending.clone()));
        }

        updates
    }

    /// Time updates, and whether the battle ended by time.
    fn time(&self) -> (Vec<Update>, bool) {
        let Some(elapsed) = self.world.elapsed() else {
            return (vec![], false);
        };
        let mut updates = vec![Update::UpdateElapsed(elapsed, self.world.duration)];

        if elapsed < self.world.duration {
            return (updates, false);
        }

        let result = self.battle_result();
        updates.push(Update::SetBattlePhase(BattlePhase::End(End::Time(result))));
        updates.push(Update::End);

        (updates, true)
    }

    #[cfg(feature = "debug")]
    pub fn force_end(&self) -> Vec<Update> {
        if matches!(self.world.phase, BattlePhase::End(_)) {
            return vec![];
        }

        let result = self.battle_result();
        vec![
            Update::SetBattlePhase(BattlePhase::End(End::Time(result))),
            Update::End,
        ]
    }

    fn battle_result(&self) -> oc_root::battle::Result {
        let morale = self.compute_morale();
        oc_root::battle::Result::from_morale(
            morale.a,
            morale.b,
            self.world.w.morale_end_threshold(),
        )
    }

    fn morale_end(&self) -> Vec<Update> {
        if self.world.phase != BattlePhase::Fight {
            return vec![];
        }

        // A side without any individual (even dead) can't be broken: nothing to measure
        let has_individuals = |side| self.world.individuals().iter().any(|i| i.side == side);
        if !has_individuals(Side::A) || !has_individuals(Side::B) {
            return vec![];
        }

        let Some(broken) = self.broken_side() else {
            return vec![];
        };
        let result = self.battle_result();

        vec![
            Update::SetBattlePhase(BattlePhase::End(End::Morale(broken, result))),
            Update::End,
        ]
    }

    fn broken_side(&self) -> Option<Side> {
        let morale = self.compute_morale();
        let threshold = self.world.w.morale_end_threshold();
        match (morale.a < threshold, morale.b < threshold) {
            (false, false) => None,
            (true, false) => Some(Side::A),
            (false, true) => Some(Side::B),
            (true, true) if morale.a <= morale.b => Some(Side::A),
            (true, true) => Some(Side::B),
        }
    }
}
