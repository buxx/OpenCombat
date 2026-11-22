use oc_projectile::{Projectile, ProjectileId};
use oc_root::{Client, geo::WorldVec2};

use crate::{
    physics::Processor,
    runner::update::{ExplosionOrigin, Update},
};

impl<'x, E: Client> Processor<'x, E> {
    /// Shell explodes where it collides (whatever it collides with). None when `projectile` is
    /// not a shell.
    pub fn shell_impact(&self, projectile: ProjectileId) -> Option<Vec<Update>> {
        let world = self.ctx.state.world();
        let Some(Projectile::Shell(shell)) = world.projectile(&projectile) else {
            return None;
        };
        let explosion = self.ctx.state._mod().shell(shell.shell).inner().clone().into();
        let position = WorldVec2::from(shell.position);
        tracing::debug!(name="physics-collision-shell", projectile=?projectile, position=?position);

        Some(vec![
            Update::SpawnExplosion(explosion, position, ExplosionOrigin::Shell(shell.shell)),
            Update::RemoveProjectile(projectile),
        ])
    }
}
