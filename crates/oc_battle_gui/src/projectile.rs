use oc_projectile::spawn::SpawnProjectiles;
use oc_root::geo::WorldVec3;

use crate::ingame::debug::projectile::SpawnProjectileProfile;

pub trait IntoSpawnProjectile {
    fn spawn(&self, start: WorldVec3, end: WorldVec3) -> SpawnProjectiles;
}

impl IntoSpawnProjectile for SpawnProjectileProfile {
    fn spawn(&self, start: WorldVec3, end: WorldVec3) -> SpawnProjectiles {
        let weapon = self.weapon;
        let ammo = self.ammunition;
        let shot = self.shot;
        let repeat = self.repeat;
        let side = self.side;
        let direction = (end - start).normalize_or_zero();
        // TODO: add inaccuracy in debug window
        SpawnProjectiles::new(
            weapon,
            ammo,
            shot,
            repeat,
            start,
            vec![direction],
            side,
            self.lag,
            None,
        )
    }
}
