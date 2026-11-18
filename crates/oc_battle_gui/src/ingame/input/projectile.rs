use bevy::prelude::*;
use oc_projectile::{Projectile, ProjectileId};

#[derive(Debug, Event)]
pub struct InsertProjectileEvent(pub ProjectileId, pub Projectile); // id, object, just fired (not sent by refresh)
