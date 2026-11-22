// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

use oc_geo::{
    Geo, UpdateGeo,
    region::{Region, WorldRegionIndex},
    tile::WorldTileIndex,
};
use oc_mod::{Mod, nature::Traversability};
use oc_physics::{Force, IgnoreSide, Physic, UpdatePhysic, collision::Material, volume::Volume};
use oc_root::{WorldConfig, geo::WorldVec3, ids::Ids, material::MaterialKind};
use oc_utils::{collections::WithIds, d2::Direction};

use crate::{bullet::Bullet, shell::Shell};

/// Same code for each projectile kind (all kinds share the same physic fields)
macro_rules! each {
    ($self:expr, $p:ident => $e:expr) => {
        match $self {
            Projectile::Bullet($p) => $e,
            Projectile::Shell($p) => $e,
        }
    };
}

// FIXME: projectiles hould have an forced life end.
// As we don't compute la perte de vitesse et donc la chute,
// une balle tiré en l'air avancera vers le ciel indéfiniement
// ou alors on calcule la chute et le ralentissement ...
pub mod bullet;
pub mod network;
pub mod shell;
pub mod spawn;

oc_utils::index_type! {
    pub struct ProjectileId(pub u64);
}

impl From<&ProjectileId> for ProjectileId {
    fn from(value: &ProjectileId) -> Self {
        *value
    }
}

#[oc_derive::archivable]
#[derive(Clone, Debug, PartialEq)]
pub enum Projectile {
    Bullet(Bullet),
    Shell(Shell),
}

impl Projectile {
    pub fn position(&self) -> WorldVec3 {
        each!(self, p => p.position)
    }

    pub fn tile(&self) -> WorldTileIndex {
        each!(self, p => p.tile)
    }

    pub fn origin(&self) -> WorldVec3 {
        each!(self, p => p.origin)
    }

    pub fn shooter(&self) -> Option<oc_individual::IndividualIndex> {
        each!(self, p => p.shooter)
    }
}

impl Region for Projectile {
    fn region(&self) -> WorldRegionIndex {
        each!(self, p => p.region)
    }

    fn set_region(&mut self, value: WorldRegionIndex) {
        each!(self, p => p.region = value)
    }
}

impl Physic for Projectile {
    fn position(&self, _: &WorldConfig) -> WorldVec3 {
        each!(self, p => p.position)
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        each!(self, p => &p.forces)
    }

    fn volumes(
        &self,
        ref_: WorldVec3,
        _: bool,
        _: &WorldConfig,
        _mod_: &Mod,
    ) -> Vec<(Volume, Traversability, Direction)> {
        vec![(
            Volume::Point {
                x: ref_.x,
                y: ref_.y,
                z: ref_.z,
            },
            Traversability::all(),
            Direction::IDENTITY,
        )]
    }

    fn ignore_side(&self) -> IgnoreSide {
        each!(self, p => IgnoreSide::Side(p.side))
    }

    fn side(&self) -> Option<oc_root::side::Side> {
        None
    }

    fn emit_proximity(&self) -> bool {
        true
    }

    fn collision_ignore_origin(&self) -> Option<WorldVec3> {
        each!(self, p => Some(p.origin))
    }
}

impl UpdatePhysic for Projectile {
    fn set_position(&mut self, value: WorldVec3) {
        each!(self, p => p.position = value)
    }

    fn push_force(&mut self, value: Force) {
        each!(self, p => p.forces.push(value))
    }

    fn remove_force(&mut self, value: &Force) {
        each!(self, p => p.forces.retain(|f| f != value))
    }

    fn set_volumes(&self, _value: Vec<(Volume, Traversability)>) {
        // Never need to update projectile volume
    }
}

impl Geo for Projectile {
    fn tile(&self) -> WorldTileIndex {
        each!(self, p => p.tile)
    }
}

impl UpdateGeo for Projectile {
    fn set_tile(&mut self, value: WorldTileIndex) {
        each!(self, p => p.tile = value)
    }
}

impl Material for Projectile {
    fn kind(&self) -> Option<MaterialKind> {
        Some(MaterialKind::Projectile)
    }
}

impl<'a> WithIds<ProjectileId, &'a Projectile> for Vec<(&'a ProjectileId, &'a Projectile)> {
    fn with_ids(&self) -> Vec<(ProjectileId, &'a Projectile)> {
        self.iter()
            .map(|(i, projectile)| (**i, *projectile))
            .collect()
    }
}

pub trait NextProjectileId {
    fn next_projectile_id(&self) -> ProjectileId;
}

impl NextProjectileId for Ids {
    fn next_projectile_id(&self) -> ProjectileId {
        ProjectileId(
            self.projectiles
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        )
    }
}
