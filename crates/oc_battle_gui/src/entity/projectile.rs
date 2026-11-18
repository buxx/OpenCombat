use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct ProjectileId(pub oc_projectile::ProjectileId);

impl AsRef<oc_projectile::ProjectileId> for ProjectileId {
    fn as_ref(&self) -> &oc_projectile::ProjectileId {
        &self.0
    }
}
