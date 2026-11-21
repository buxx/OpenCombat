use derive_more::Constructor;
use oc_individual::{Individual, IndividualIndex, Status, Update};
use oc_root::physics::Meters;
use oc_world::World;

use crate::{
    index::Indexes,
    individual::situation::{Situation, Visible},
    runner,
};

mod accomplish;
mod act;
pub mod death;
mod decide;
mod distribute;
mod embarked;
mod gesture;
pub mod situation;
#[cfg(test)]
mod tests;
pub mod update;
mod weapon;

const POSITION_TOLERANCE: f32 = 3.0;

// Tolerance to consider a path step reached. Must stay under the navmesh agent radius (2.5px), or
// individuals cut corners and enter walls.
const PATH_STEP_TOLERANCE: f32 = 1.5;

#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
    index: &'a Indexes,
    i: IndividualIndex,
}

impl<'a> Processor<'a> {
    /// Compute individual changes.
    ///
    /// Individual behavior is splitted in several steps:
    ///   - resolve order into intent (`decide` function): Order can be follow or not. For example,
    ///     if order is to engage an enemy but individual is suppressed, intent will be hide.
    ///   - resolve an intent into behavior (`act` function): Here the intent can be specialized in
    ///     behavior to reach the intent goal. For example, a move intent will be splitted in move
    ///     behavior which target the next move step instead move target.
    ///   - resolve behavior in gesture: body and hands gesture are computed from the behavior and
    ///     individual state. For example, engage behavior can result a reloading hands, or aiming
    ///     hands.
    ///   - resolve forces from behavior: Here, behavior forces are computed. For example, a move
    ///     behavior result a translation force.
    ///
    pub fn step(self) -> Vec<runner::update::Update> {
        tracing::trace!(name="individual-step", i=?self.i);
        let mut updates = vec![];
        let individual = self.world.individual(self.i);

        if !individual.status.can_step() {
            tracing::trace!(name="individual-step-cant-step", i=?self.i);
            return vec![];
        }

        self.stunned(&mut updates, individual);
        self.suppress(&mut updates, individual);

        if !individual.can_act() {
            tracing::trace!(name="individual-step-cant-act", i=?self.i);
            return updates;
        }

        let situation = &self.situation(individual);

        if let Some(updates) = self.accomplished(situation) {
            tracing::trace!(name = "individual-step-accomplished-updates", i=?self.i, updates=?updates);
            return updates;
        }

        let distribute = self.distribute();

        // Individual in vehicle operates it instead of acting by itself
        if let Some((vehicle, role)) = individual.embarked {
            updates.extend(distribute);
            updates.extend(self.embarked(individual, vehicle, role));
            tracing::trace!(name = "individual-step-embarked-updates", i=?self.i, updates=?updates);
            return updates;
        }

        let intent = self.decide(situation);
        let behavior = self.act(situation, &intent);
        let (gesture, updates_) = self.gesture(individual, situation, &behavior);
        let forces = self.forces(individual, &behavior, &intent);
        #[cfg(feature = "battle_instrument")]
        self.instrument_reverse(individual, &intent, &behavior);

        tracing::trace!(
            name = "individual-step-with",
            i = ?self.i,
            individual = ?individual,
            distribute = ?distribute,
            intent = ?intent,
            behavior = ?behavior,
            gesture = ?gesture,
            forces = ?forces,
        );

        updates.extend(distribute);
        updates.extend(updates_);

        macro_rules! push_update {
            ($self:expr, $updates:expr, $individual:expr, $( $field:ident => $variant:ident ),+ $(,)?) => {
                $(
                    if $field != $individual.$field {
                        $updates.push(runner::update::Update::UpdateIndividual(
                            $self.i,
                            Update::$variant($field),
                        ));
                    }
                )+
            };
        }

        push_update!(self, updates, individual,
            intent => SetIntent,
            behavior => SetBehavior,
            gesture => SetGesture,
            forces => SetForces,
        );

        tracing::trace!(name = "individual-step-updates", i=?self.i, updates=?updates);
        updates
    }

    fn stunned(&self, updates: &mut Vec<runner::update::Update>, individual: &Individual) {
        let Status::Stunned(stun) = individual.status else {
            return;
        };
        let stun = stun.decrease(self.world.w.individual_tick_decrease_stunned());
        let status = if stun.is_zero() {
            Status::Operational
        } else {
            Status::Stunned(stun)
        };

        updates.push(runner::update::Update::UpdateIndividual(
            self.i,
            Update::SetStatus(status),
        ));
    }

    fn suppress(&self, updates: &mut Vec<runner::update::Update>, individual: &Individual) {
        let decrease = self.world.w.individual_tick_decrease_suppress();
        let suppress_ = individual.suppress;
        let suppress = individual.suppress.decrease(decrease);
        if suppress_ != suppress {
            updates.push(runner::update::Update::UpdateIndividual(
                self.i,
                oc_individual::Update::SetSuppress(suppress),
            ));
        }
    }

    /// Build object which reflect individual situation against environment
    fn situation(&self, individual: &Individual) -> Situation<'_> {
        let reference = individual.position;

        let mut visibles: Vec<Visible> = self
            .world
            .visibilities
            .for_(self.i)
            .iter()
            .enumerate()
            .filter(|&(_i, v)| v.visible)
            .filter(|&(i, _v)| {
                self.world
                    .individual(IndividualIndex::from(i))
                    .status
                    .is_target()
            })
            .map(|(i, v)| {
                let target = IndividualIndex::from(i);
                let target_ = self.world.individual(target);
                let distance = reference.distance(target_.position);
                let distance = Meters(distance / self.world.w.geo_pixels_per_meters());
                Visible::new(target, v, distance)
            })
            .collect();
        visibles.sort_unstable_by(|a, b| a.distance.0.total_cmp(&b.distance.0));

        Situation { visibles }
    }
}
