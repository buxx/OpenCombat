use crate::{Projectile as Projectile_, ProjectileId};

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum Projectile {
    Insert(ProjectileId, Projectile_),
    Physics(ProjectileId, oc_physics::update::Update),
    Forgot(ProjectileId),
}
