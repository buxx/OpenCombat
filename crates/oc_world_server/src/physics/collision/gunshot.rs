use oc_individual::IndividualIndex;
use oc_projectile::ProjectileId;
use oc_root::Client;

use crate::{individual::death::die, physics::Processor, runner::update::Update};

impl<'x, E: Client> Processor<'x, E> {
    pub fn gunshot(&self, projectile: ProjectileId, i: IndividualIndex) -> Vec<Update> {
        let mut updates = vec![];
        #[cfg(feature = "battle_instrument")]
        let Some(projectile_) = self.ctx.state.world().projectile(&projectile).cloned() else {
            return updates;
        };
        #[cfg(not(feature = "battle_instrument"))]
        let Some(_) = self.ctx.state.world().projectile(&projectile) else {
            return updates;
        };
        let world = self.ctx.state.world();
        let individual = world.individual(i);
        tracing::trace!(name="physics-collision-gunshot", projectile=?projectile, individual=?i);

        if !individual.status.is_dead() {
            #[cfg(feature = "tracker")]
            {
                self.ctx.tracker.take().individuals.push((
                    i,
                    oc_individual::Update::SetStatus(oc_individual::Status::Dead),
                ));
            }
            #[cfg(feature = "battle_instrument")]
            {
                let oc_projectile::Projectile::Bullet(bullet) = &projectile_;
                crate::battle_log::kill(
                    &world,
                    i,
                    crate::battle_log::Cause::Gunshot(bullet.shooter, bullet.origin.into()),
                );
            }
            updates.extend(die(&world, self.ctx.state._mod(), i));
            updates.push(Update::RemoveProjectile(projectile));
        }

        updates
    }
}
