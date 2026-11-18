use std::time::{Duration, Instant};

use derive_more::Constructor;
use oc_mod::Mod;
use oc_physics::Force;
use oc_projectile::{
    Projectile,
    bullet::Bullet,
    spawn::{SpawnProjectile, SpawnProjectiles},
};
use oc_root::WorldConfig;

use crate::schedule::{Schedule, Scheduling};

#[derive(Debug, Constructor)]
pub struct Builder<'a, 'b> {
    w: &'b WorldConfig,
    mod_: &'a Mod,
    spawn: SpawnProjectile,
}

impl<'a, 'b> Builder<'a, 'b> {
    pub fn build(&self) -> Projectile {
        let weapon = self.mod_.weapon(self.spawn.weapon);
        let ammunition = self.mod_.ammunition(self.spawn.ammunition);
        let from = self.spawn.from;
        let direction = self.spawn.direction;
        let speed = weapon.velocity();
        let forces = vec![Force::Translation(direction, speed)];

        match ammunition {
            oc_mod::ammunition::Ammunition::Cartridge(_) => Projectile::Bullet(Bullet::new(
                from,
                self.spawn.side,
                forces,
                self.spawn.shooter,
                self.w,
            )),
        }
    }
}

impl Schedule<(&WorldConfig, &Mod), (Instant, bool)> for SpawnProjectiles {
    fn schedule(&self, (w, mod_): (&WorldConfig, &Mod)) -> Scheduling<(Instant, bool)> {
        let weapon = mod_.weapon(self.weapon);
        let repeat = self.repeat;
        let shot = weapon.shot(self.shot);
        let rounds = shot.rounds();
        let mut instant = Instant::now() + self.lag;
        let mut instants = vec![];

        for _ in 0..repeat {
            let mut fx = true;
            for _ in 0..rounds {
                instants.push((instant, fx));
                instant += Duration::from_millis((shot.interval().0 / w.speed() * 1000.0) as u64);
                fx = false;
            }

            instant += Duration::from_millis((weapon.interval().0 / w.speed() * 1000.0) as u64);
        }

        Scheduling(instants)
    }
}
