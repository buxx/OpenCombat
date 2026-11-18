use oc_geo::tile::WorldTileIndex;
use oc_individual::{
    Individual,
    behavior::{Behavior, Intent, MovePath},
};
use oc_mod::nature::{IndividualMovement, MovementForce};
use oc_physics::Force;
use oc_root::WcfgFrom;
use oc_root::geo::{WorldVec2, WorldVec3};
use oc_utils::d2::Direction;

use super::{Processor, situation::Situation};

impl Processor<'_> {
    // TODO: Ex. idle -> Behavior::TakingCover when under fire
    pub fn act(&self, _situation: &Situation, intent: &Intent) -> Behavior {
        let individual = self.world.individual(self.i);

        let behavior = match intent {
            Intent::Idle(direction) => Behavior::Idle(*direction),
            Intent::MoveTo(_, path) => self.move_behavior(individual, path, "move", Behavior::Walk),
            Intent::MoveFastTo(_, path) => {
                self.move_behavior(individual, path, "move-fast", Behavior::Run)
            }
            Intent::SneakTo(_, path) => {
                self.move_behavior(individual, path, "sneak", Behavior::Crawl)
            }
            Intent::Defend(direction) => Behavior::Defend(*direction),
            Intent::Hide(direction) => Behavior::Hide(*direction),
            Intent::Engage(target) => Behavior::Engage(*target),
            Intent::Suppress(position) => Behavior::Suppress(*position),
            Intent::Grenade(position) => Behavior::Grenade(*position),
            Intent::Smoke(position) => Behavior::Smoke(*position),
        };

        match self.world.phase.fight() {
            true => self.bahavior_modified_by_tile(individual, behavior),
            false => self.act_direction(individual, behavior),
        }
    }

    fn move_behavior(
        &self,
        individual: &Individual,
        path: &MovePath,
        trace_name: &'static str,
        movement: fn(Direction) -> Behavior,
    ) -> Behavior {
        match self.direction_to_next(individual, path, trace_name) {
            Some(direction) => movement(direction),
            None => Behavior::Idle(Direction::NORTH),
        }
    }

    fn act_direction(&self, individual: &Individual, behavior: Behavior) -> Behavior {
        let from = individual.position;
        match behavior {
            Behavior::Idle(direction)
            | Behavior::Walk(direction)
            | Behavior::Run(direction)
            | Behavior::Crawl(direction)
            | Behavior::Defend(direction)
            | Behavior::Hide(direction) => Behavior::Idle(direction),
            Behavior::Engage(target) => {
                let target = self.world.individual(target);
                Behavior::Idle(Direction::from_points2d(
                    from.to_vec2().into(),
                    target.position.to_vec2().into(),
                ))
            }
            Behavior::Suppress(position)
            | Behavior::Grenade(position)
            | Behavior::Smoke(position) => Behavior::Idle(Direction::from_points2d(
                from.to_vec2().into(),
                position.into(),
            )),
        }
    }

    fn bahavior_modified_by_tile(&self, individual: &Individual, behavior: Behavior) -> Behavior {
        let direction = match &behavior {
            Behavior::Walk(direction) | Behavior::Run(direction) | Behavior::Crawl(direction) => {
                *direction
            }
            // Other behavior than movement are keep as it
            _ => return behavior,
        };

        let force = self.tile_movement(individual).force;
        // Forced behavior only replaces behaviors "superior" (faster) or equal to it
        if !Self::tile_force_applies(force, &behavior) {
            return behavior;
        }

        match force {
            MovementForce::None => behavior,
            MovementForce::Sneak => Behavior::Crawl(direction),
            MovementForce::Walk => Behavior::Walk(direction),
            MovementForce::Run => Behavior::Run(direction),
        }
    }

    /// Tile movement force concerns the behavior only if it is "superior" (faster) or equal to it.
    fn tile_force_applies(force: MovementForce, behavior: &Behavior) -> bool {
        matches!(
            (force, behavior),
            (MovementForce::Sneak, _)
                | (MovementForce::Walk, Behavior::Walk(_) | Behavior::Run(_))
                | (MovementForce::Run, Behavior::Run(_))
        )
    }

    fn tile_movement(&self, individual: &Individual) -> IndividualMovement {
        let w = &self.world.w;
        let tile_i = WorldTileIndex::from_(WorldVec2::from(individual.position), w);
        match self.world.tile(tile_i) {
            Some(tile) => self.world.mod_.nature(tile.nature).movement.individual,
            None => IndividualMovement::default(),
        }
    }

    fn direction_to_next(
        &self,
        individual: &Individual,
        path: &MovePath,
        trace_name: &'static str,
    ) -> Option<Direction> {
        let Some(next) = path.iter().next() else {
            tracing::trace!(name = format!("individual-step-act-{trace_name}-no-next"), i=?self.i);
            return None;
        };

        let from = WorldVec2::from(individual.position);
        let direction = (*next - from).normalize_or_zero();
        Some(Direction::from(direction))
    }

    /// Compute forces for the current behavior.
    pub fn forces(
        &self,
        individual: &Individual,
        behavior: &Behavior,
        _intent: &Intent,
    ) -> Vec<Force> {
        match behavior {
            Behavior::Walk(direction) | Behavior::Run(direction) | Behavior::Crawl(direction) => {
                let nominal_speed = behavior.nominal_speed(&self.world.w);
                // TODO: here use fatigue, injures, etc
                // Tile factor applies in the same condition than the tile behavior modification
                let tile_movement = self.tile_movement(individual);
                let move_disability_factor =
                    if Self::tile_force_applies(tile_movement.force, behavior) {
                        tile_movement.factor
                    } else {
                        1.0
                    };
                let direction = WorldVec3::new(direction.x, direction.y, 0.);
                vec![Force::Translation(
                    direction,
                    nominal_speed * move_disability_factor,
                )]
            }
            Behavior::Idle(_)
            | Behavior::Defend(_)
            | Behavior::Hide(_)
            | Behavior::Engage(_)
            | Behavior::Suppress(_)
            | Behavior::Grenade(_)
            | Behavior::Smoke(_) => vec![],
        }
    }
}
