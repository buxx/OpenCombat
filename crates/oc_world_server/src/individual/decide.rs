use glam::Vec2;
use oc_individual::{
    Individual, IndividualIndex,
    behavior::{Intent, MovePath, THROW_MAX_DISTANCE},
    order::Order,
};
use oc_mod::grenade::GrenadeNature;
use oc_root::geo::WorldVec2;
use oc_utils::d2::Direction;

use super::{Processor, situation::Situation};

// AI generated
// When individual has no path from its position (see `resolve_path`), look for path from points
// around it, until this radius (pixels).
const PATH_UNSTUCK_MAX_RADIUS: usize = 5;
const PATH_UNSTUCK_DIRECTIONS: [(f32, f32); 8] = [
    (1., 0.),
    (-1., 0.),
    (0., 1.),
    (0., -1.),
    (0.7, 0.7),
    (-0.7, 0.7),
    (0.7, -0.7),
    (-0.7, -0.7),
];

/// Factor to move to position hwere can thro grenade (to ensure near enough)
const THROW_REACH_FACTOR: f32 = 0.95;

type MoveIntent = fn(WorldVec2, MovePath) -> Intent;

impl Processor<'_> {
    /// Decide the individual's intent for this tick. Will be influenced by situation.
    pub fn decide(&self, situation: &Situation) -> Intent {
        let individual = self.world.individual(self.i);
        let order = individual.orders.first();

        let intent = match individual.can_follow_orders() {
            // FIXME BS NOW: suppress ?
            true => match order {
                None | Some(Order::Idle) => self.resolve_idle_order(individual, situation),
                Some(Order::Defend(direction)) => {
                    self.resolve_defend_order(individual, situation, *direction)
                }
                Some(Order::Hide(direction)) => {
                    self.resolve_hide_order(individual, situation, *direction)
                }
                Some(Order::MoveTo(position)) => {
                    // Move fast as enemy can fire on them
                    // FIXME BS NOW: must move fast only if direction do not is direction of enemy !
                    let make = match situation.visibles.is_empty() {
                        true => Intent::MoveTo as MoveIntent,
                        false => Intent::MoveFastTo as MoveIntent,
                    };
                    self.resolve_move_order(individual, *position, make)
                }
                Some(Order::MoveFastTo(position)) => {
                    self.resolve_move_order(individual, *position, Intent::MoveFastTo)
                }
                Some(Order::SneakTo(position)) => {
                    self.resolve_move_order(individual, *position, Intent::SneakTo)
                }
                Some(Order::Engage(target)) => {
                    self.resolve_engage_order(individual, situation, *target)
                }
                Some(Order::Suppress(target)) => self.resolve_suppress_order(individual, *target),
                Some(Order::Grenade(target)) => {
                    self.resolve_throw_order(individual, *target, Intent::Grenade)
                }
                Some(Order::Smoke(target)) => {
                    self.resolve_throw_order(individual, *target, Intent::Smoke)
                }
            },
            // Dead (or unable) individuals do nothing: no movement intent is kept
            false => Intent::Idle(individual.gesture.direction()),
        };

        tracing::trace!(name="individual-step-decide", i=?self.i, order=?order, intent=?intent);
        intent
    }

    /// When suppressed enough, individual hides whatever its order is.
    fn suppressed_hide(&self, individual: &Individual) -> Option<Intent> {
        match individual.suppress >= self.world.w.individual_suppress_limit_hide() {
            true => {
                tracing::trace!(name="individual-processor-resolve-order-suppressed", i=?self.i);
                Some(Intent::Hide(individual.gesture.direction()))
            }
            false => None,
        }
    }

    pub fn resolve_idle_order(&self, individual: &Individual, situation: &Situation) -> Intent {
        tracing::trace!(name="individual-processor-resolve-idle-order", i=?self.i);
        let direction = individual.gesture.direction();

        if let Some(intent) = self.suppressed_hide(individual) {
            return intent;
        }

        if let Some(intent) = self.grenade_on_near_enemy(individual, situation) {
            return intent;
        }

        // FIXME: introduce suppressed, etc (refactored way!)
        match &individual.intent {
            // If already engaging, continue try to engage this target
            Intent::Engage(target) => self.engage_intent(situation, *target, direction),
            // TODO: something somewhere to prevent all squad members fire same individual (because nearest) ?
            _ => match situation.visibles.first() {
                Some(visible) => self.engage_intent(situation, visible.individual, direction),
                None => Intent::Idle(direction),
            },
        }
    }

    /// Individual owning a frag grenade throws it on the nearest visible enemy when it is near
    /// enough (see `individual_grenade_enemy_distance`).
    fn grenade_on_near_enemy(
        &self,
        individual: &Individual,
        situation: &Situation,
    ) -> Option<Intent> {
        self.grenade_of(individual, GrenadeNature::Frag)?;

        let near = self.world.w.individual_grenade_enemy_distance();
        let visible = situation.visibles.first().filter(|v| v.distance <= near)?;
        let enemy = self.world.individual(visible.individual);
        tracing::trace!(name="individual-processor-grenade-on-near-enemy", i=?self.i, enemy=?visible.individual);

        Some(Intent::Grenade(WorldVec2::from(enemy.position)))
    }

    fn engage_intent(
        &self,
        situation: &Situation,
        target: IndividualIndex,
        direction: Direction,
    ) -> Intent {
        match situation.visible(target).is_some()
            && self.world.individual(target).status.is_target()
        {
            true => Intent::Engage(target),
            false => Intent::Idle(direction),
        }
    }

    pub fn resolve_defend_order(
        &self,
        individual: &Individual,
        situation: &Situation,
        direction: Direction,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-defend-order", i=?self.i);

        if let Some(intent) = self.suppressed_hide(individual) {
            return intent;
        }

        if let Some(intent) = self.grenade_on_near_enemy(individual, situation) {
            return intent;
        }

        // FIXME: introduce suppressed, etc (refactored way!)
        match &individual.intent {
            // If already engaging, continue only if target is visible and not dead
            Intent::Engage(target)
                if situation.visible(*target).is_some()
                    && self.world.individual(*target).status.is_target() =>
            {
                Intent::Engage(*target)
            }
            _ => match situation.visibles.first() {
                Some(visible) => Intent::Engage(visible.individual),
                None => Intent::Defend(direction),
            },
        }
    }

    fn resolve_hide_order(
        &self,
        individual: &Individual,
        situation: &Situation,
        direction: Direction,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-hide-order", i=?self.i);
        let w = &self.world.w;

        if let Some(intent) = self.suppressed_hide(individual) {
            return intent;
        }

        if let Some(intent) = self.grenade_on_near_enemy(individual, situation) {
            return intent;
        }

        // If currently engaging, continue it only if individual is in near area
        if let Intent::Engage(target) = &individual.intent
            && situation
                .visible(*target)
                .is_some_and(|v| v.distance <= w.individual_hide_engage_distance())
            && self.world.individual(*target).status.is_target()
        {
            return Intent::Engage(*target);
        }

        // FIXME: this will engage first soldier entering in area. So other ennemi=y soldiers will hide and
        // not enter in zone. Add a second area which permit to continue to engage (but how to do that ...?)
        match situation
            .visibles
            .iter()
            .find(|v| v.distance <= w.individual_hide_engage_distance())
        {
            Some(visible) => Intent::Engage(visible.individual),
            None => Intent::Hide(direction),
        }
    }

    /// Move (or sneak) order: `make` builds the intent from the position and the path to follow.
    // FIXME BS NOW: sneak if tired (and can't run)
    // FIXME BS NOW: impact of suppress ?
    fn resolve_move_order(
        &self,
        individual: &Individual,
        position: WorldVec2,
        make: MoveIntent,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-move-order", i=?self.i);

        if let Some(intent) = self.suppressed_hide(individual) {
            return intent;
        }

        match self.resolve_path(individual, position) {
            Some(path) => make(position, path),
            None => {
                tracing::trace!(name="individual-processor-resolve-move-order-no-path", i=?self.i);
                let reference = individual.position.into();
                let target: Vec2 = position.into();
                let direction = Direction::from_points3d(reference, target.extend(0.));
                Intent::Hide(direction)
            }
        }
    }

    // FIXME BS NOW: impact of suppress ?
    // FIXME BS NOW: impact of no ammo / weapon for target ?
    fn resolve_engage_order(
        &self,
        individual: &Individual,
        situation: &Situation,
        target: IndividualIndex,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-engage-order", i=?self.i, target=?target);

        if let Some(intent) = self.suppressed_hide(individual) {
            return intent;
        }

        // Target is visible, engage it
        if situation.visible(target).is_some() && self.world.individual(target).status.is_target() {
            return Intent::Engage(target);
        }

        // Target is no more engageable: try to find another one
        match self.squad_target(individual) {
            Some(other) => Intent::Engage(other),
            None => {
                // Use original target as direction
                let target = self.world.individual(target);
                let reference = individual.position.into();
                let direction = Direction::from_points3d(reference, target.position.into());
                Intent::Hide(direction)
            }
        }
    }

    /// Nearest (from individual) target which is visible by at least one squad member and engageable.
    fn squad_target(&self, individual: &Individual) -> Option<IndividualIndex> {
        let squad = self.world.squad(self.index.individual_squad(self.i));

        squad
            .members
            .iter()
            .flat_map(|member| {
                self.world
                    .visibilities
                    .for_(*member)
                    .iter()
                    .enumerate()
                    .filter(|&(_, v)| v.visible)
                    .map(|(i, _)| IndividualIndex::from(i))
            })
            .filter(|target| self.world.individual(*target).status.is_target())
            .map(|target| {
                let target_ = self.world.individual(target);
                (target, individual.position.distance(target_.position))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(target, _)| target)
    }

    // FIXME BS NOW: impact of suppress ?
    fn resolve_suppress_order(&self, individual: &Individual, target: WorldVec2) -> Intent {
        tracing::trace!(name="individual-processor-resolve-suppress-order", i=?self.i);

        if let Some(intent) = self.suppressed_hide(individual) {
            return intent;
        }

        Intent::Suppress(target)
    }

    fn resolve_throw_order(
        &self,
        individual: &Individual,
        target: WorldVec2,
        intent: fn(WorldVec2) -> Intent,
    ) -> Intent {
        tracing::trace!(name="individual-processor-resolve-throw-order", i=?self.i);

        if let Some(intent) = self.suppressed_hide(individual) {
            return intent;
        }

        // Too far: move to the nearest position from which target is in range
        let w = &self.world.w;
        let max = THROW_MAX_DISTANCE.pixels(w);
        let from: Vec2 = individual.position.into();
        let to: Vec2 = target.into();
        if from.distance(to) > max {
            let reach = to + (from - to).normalize() * max * THROW_REACH_FACTOR;
            let reach = WorldVec2::new(reach.x, reach.y);
            tracing::trace!(name="individual-processor-resolve-throw-order-too-far", i=?self.i, reach=?reach);
            return self.resolve_move_order(individual, reach, Intent::MoveFastTo);
        }

        intent(target)
    }

    /// Compute path to target, or reuse current if already known path
    fn resolve_path(&self, individual: &Individual, target: WorldVec2) -> Option<MovePath> {
        if let Some((current_target, current_path)) = individual.intent.path()
            && current_target == target
            && current_path.iter().next().is_some()
        {
            return Some(current_path.clone());
        }

        let from = (individual.position.x, individual.position.y);
        let to = (target.x, target.y);
        if let Some(path) = self.world.navmesh.path(from, to) {
            return Some(path.into());
        }

        // AI generated (maybe found another solution)
        // Individual can be stuck against a wall (physics stops it on the wall edge, which is out
        // of the navmesh because of the agent radius): look for a path from a nearby free point
        // to let it go out.
        for radius in 1..=PATH_UNSTUCK_MAX_RADIUS {
            for (dx, dy) in PATH_UNSTUCK_DIRECTIONS {
                let from = (from.0 + dx * radius as f32, from.1 + dy * radius as f32);
                if let Some(path) = self.world.navmesh.path(from, to) {
                    tracing::trace!(name="individual-resolve-path-unstuck", i=?self.i, from=?from);
                    return Some(path.into());
                }
            }
        }

        None
    }
}
