// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

use oc_utils::index::AtIndex;
use oc_utils::percent::Percent;
use std::time::{Duration, Instant};

use derive_more::Constructor;
use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{
    Individual, IndividualIndex,
    squad::{Squad, SquadIndex},
};
use oc_mod::Mod;
use oc_projectile::{Projectile, ProjectileId};
use oc_root::{
    WcfgInto, WorldConfig,
    battle::BattlePhase,
    identity::Identity,
    material::MaterialKind,
    side::{BySide, Side},
    uuids::Uuids,
};
use oc_utils::{d2::Xy, let_some};
use oc_vehicle::{Vehicle, VehicleIndex};
use rustc_hash::FxHashMap;

use crate::{
    crater::Crater,
    decor::Decor,
    flag::Flags,
    interior::Interior,
    meta::Meta,
    navmesh::Walls,
    resume::WorldResume,
    smoke::SmokeCloud,
    support::{PendingSupport, SupportType, Supports},
    tile::AlteredTile,
    tile::Tile,
    visibility::Visibilities,
    wreck::Wreck,
};

pub mod cache;
pub mod check;
pub mod control;
pub mod crater;
pub mod decor;
pub mod flag;
pub mod glb;
pub mod interior;
pub mod load;
pub mod map;
pub mod meta;
pub mod morale;
pub mod navmesh;
pub mod place;
pub mod reader;
pub mod resume;
pub mod smoke;
pub mod snapshot;
pub mod spawn;
pub mod support;
pub mod terrain;
pub mod tile;
pub mod utils;
pub mod visibility;
pub mod wreck;

#[derive(Constructor)]
#[allow(clippy::too_many_arguments)]
pub struct World {
    pub w: WorldConfig,
    pub mod_: Mod, // Maybe its place is not in world (when want to access, need to read lock World, but Mod never change)
    pub meta: Meta,
    pub phase: BattlePhase,
    pub duration: Duration,
    pub start: Option<Instant>,
    pub side_spawn_zones: BySide<Vec<crate::spawn::SpawnZone>>,
    pub spawn_zones: Vec<crate::spawn::SpawnZone>,
    pub flags: Flags,
    pub supports: Supports,
    pub pending_supports: Vec<PendingSupport>,
    pub support_realized: FxHashMap<(Side, SupportType), Instant>,
    pub tiles: Vec<Tile>,
    pub altered_tiles: Vec<Option<AlteredTile>>,
    pub decor: Vec<Option<Decor>>,
    pub craters: Vec<Crater>,
    pub wrecks: Vec<Wreck>,
    pub smoke: Vec<Percent>,
    pub smoke_clouds: Vec<SmokeCloud>,
    pub navmesh: polyanya::Mesh,
    pub visibilities: Visibilities,
    pub individuals: Vec<Individual>,
    pub squads: Vec<Squad>,
    pub vehicles: Vec<Vehicle>,
    pub projectiles: FxHashMap<ProjectileId, Projectile>,
    pub interiors: Vec<Interior>,
    pub places: Vec<crate::place::Place>,
    pub uuids: Uuids,
}

impl World {
    /// Spawn zones owned by given side
    pub fn spawn_zones(&self, side: Side) -> &[crate::spawn::SpawnZone] {
        &self.side_spawn_zones[side]
    }

    pub fn tile(&self, i: WorldTileIndex) -> Option<&Tile> {
        self.tiles.get_at(i)
    }

    // TODO: unit test me
    pub fn region_tiles(&self, region: WorldRegionIndex) -> Vec<(WorldTileIndex, &Tile)> {
        let region_: RegionXy = region.into_(&self.w);
        let start: TileXy = region_.into_(&self.w);
        let count = self.w.region_width() * self.w.region_height();
        let mut tiles = Vec::with_capacity(count as usize);

        tracing::debug!("Extract region {} tiles", region.0,);
        for y in 0..self.w.region_height() {
            let line_start = TileXy(Xy(start.0.0, start.0.1 + y));
            let line_start: WorldTileIndex = line_start.into_(&self.w);
            let line_start = line_start.0;
            let line_end = line_start + self.w.region_width();
            let tiles_ = &self.tiles[line_start as usize..line_end as usize];
            let tiles_: Vec<(WorldTileIndex, &Tile)> = tiles_
                .iter()
                .zip(line_start..line_end)
                .map(|(t, i)| (WorldTileIndex(i), t))
                .collect();
            tiles.extend(tiles_);
        }

        tiles
    }

    pub fn alter_tiles(&mut self, altered: Vec<AlteredTile>) {
        let mod_ = &self.mod_;
        let mut walls_changed = false;

        for altered in altered {
            let i = altered.i.0 as usize;
            let_some!(tile = self.tiles.get_mut(i), continue);
            let was_wall = tile.allow.deny(MaterialKind::Individual);
            tile.alter(&altered, mod_);
            walls_changed |= was_wall != tile.allow.deny(MaterialKind::Individual);
            self.altered_tiles[i] = Some(altered);
        }

        if walls_changed {
            let walls = self.tiles.as_walls(mod_);
            self.navmesh = navmesh::navmesh(&self.w, &walls);
        }
    }

    pub fn region_altered_tiles(&self, region: WorldRegionIndex) -> Vec<AlteredTile> {
        self.region_tiles(region)
            .into_iter()
            .filter_map(|(i, _)| self.altered_tiles.at(i).clone())
            .collect()
    }

    pub fn region_decor(&self, region: WorldRegionIndex) -> Vec<Decor> {
        self.region_tiles(region)
            .into_iter()
            .filter_map(|(i, _)| self.decor.at(i).clone())
            .collect()
    }

    pub fn region_wrecks(&self, region: WorldRegionIndex) -> Vec<Wreck> {
        self.wrecks
            .iter()
            .filter(|wreck| {
                let region_: WorldRegionIndex = wreck.tile(&self.w).into_(&self.w);
                region_ == region
            })
            .cloned()
            .collect()
    }

    /// Add a wreck for each destroyed vehicle
    pub fn wreck_destroyed_vehicles(&mut self) {
        let wrecks = self
            .vehicles
            .iter()
            .filter(|vehicle| vehicle.destroyed)
            .map(Wreck::from_vehicle);
        self.wrecks.extend(wrecks);
    }

    pub fn smoke_at(&self, i: WorldTileIndex) -> Percent {
        self.smoke.get_at(i).copied().unwrap_or_default()
    }

    pub fn set_smoke(&mut self, smoke: &[(WorldTileIndex, Percent)]) {
        for (i, opacity) in smoke {
            *self.smoke.at_mut(*i) = *opacity;
        }
    }

    /// Smoke clouds keep their opacity during their duration, then fade. Expired clouds are
    /// forgotten. Returns tiles whose opacity changed.
    pub fn update_smoke(&mut self, now: Instant) -> Vec<(WorldTileIndex, Percent)> {
        let mut opacities: FxHashMap<WorldTileIndex, Percent> = FxHashMap::default();

        for cloud in &self.smoke_clouds {
            for (i, opacity) in cloud.opacities(now) {
                let max = opacities.entry(i).or_default();
                *max = (*max).max(opacity);
            }
        }

        let changed: Vec<(WorldTileIndex, Percent)> = opacities
            .into_iter()
            .filter(|(i, opacity)| self.smoke_at(*i) != *opacity)
            .collect();
        self.set_smoke(&changed);
        self.smoke_clouds.retain(|cloud| !cloud.expired(now));

        changed
    }

    pub fn region_smoke(&self, region: WorldRegionIndex) -> Vec<(WorldTileIndex, Percent)> {
        self.region_tiles(region)
            .into_iter()
            .map(|(i, _)| (i, self.smoke_at(i)))
            .filter(|(_, opacity)| !opacity.is_zero())
            .collect()
    }

    pub fn remove_decor(&mut self, tiles: &[WorldTileIndex]) {
        for i in tiles {
            *self.decor.at_mut(*i) = None;
        }
    }

    pub fn individuals(&self) -> &[Individual] {
        &self.individuals
    }

    pub fn individual(&self, i: IndividualIndex) -> &Individual {
        self.individuals.at(i)
    }

    pub fn individual_mut(&mut self, i: IndividualIndex) -> &mut Individual {
        self.individuals.at_mut(i)
    }

    pub fn vehicles(&self) -> &[Vehicle] {
        &self.vehicles
    }

    pub fn vehicle(&self, i: VehicleIndex) -> &Vehicle {
        self.vehicles.at(i)
    }

    pub fn wreck(&self, i: wreck::WreckIndex) -> &wreck::Wreck {
        self.wrecks.at(i)
    }

    pub fn vehicle_mut(&mut self, i: VehicleIndex) -> &mut Vehicle {
        self.vehicles.at_mut(i)
    }

    pub fn squads(&self) -> &[Squad] {
        &self.squads
    }

    pub fn squad(&self, i: SquadIndex) -> &Squad {
        self.squads.at(i)
    }

    pub fn squad_mut(&mut self, i: SquadIndex) -> &mut Squad {
        self.squads.at_mut(i)
    }

    pub fn projectiles(&self) -> Vec<(&ProjectileId, &Projectile)> {
        self.projectiles.iter().collect()
    }

    pub fn projectiles_mut(&mut self) -> &mut FxHashMap<ProjectileId, Projectile> {
        &mut self.projectiles
    }

    pub fn projectile(&self, i: &ProjectileId) -> Option<&Projectile> {
        self.projectiles.get(i)
    }

    pub fn projectile_mut(&mut self, i: &ProjectileId) -> Option<&mut Projectile> {
        self.projectiles.get_mut(i)
    }

    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    pub fn mod_(&self) -> &Mod {
        &self.mod_
    }

    /// Supports the side can still request: counter minus already pending ones
    pub fn available_support(&self, side: Side, type_: &SupportType) -> u8 {
        let pending = self
            .pending_supports
            .iter()
            .filter(|p| p.side == side && &p.type_ == type_)
            .count();
        self.supports[side]
            .count(type_)
            .saturating_sub(pending.min(u8::MAX as usize) as u8)
    }

    pub fn resume(&self, identity: &Identity) -> WorldResume {
        let squads = self
            .squads
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut squad = s.clone();
                if s.side != identity.side {
                    // Opposite squads are sent for minimap display only: don't leak their orders
                    squad.orders = vec![];
                }
                (SquadIndex::from(i), squad)
            })
            .collect();
        let phase = self.phase;
        let support = self.supports[identity.side]
            .0
            .iter()
            .map(|(t, _)| (t.clone(), self.available_support(identity.side, t)))
            .collect();
        let spawns = self
            .side_spawn_zones
            .as_ref()
            .map(|zones| zones.iter().map(|z| z.name.clone()).collect());

        WorldResume {
            squads,
            phase,
            spawns,
            support,
        }
    }

    pub fn elapsed(&self) -> Option<Duration> {
        self.start
            .map(|start| start.elapsed().mul_f32(self.w.speed()))
    }

    pub fn set_phase(&mut self, value: BattlePhase) {
        if matches!(value, BattlePhase::Fight) && self.start.is_none() {
            self.start = Some(Instant::now());
        }
        self.phase = value;
    }

    pub fn tile_z(&self, w: &WorldConfig, i: WorldTileIndex) -> f32 {
        self.tile(i).map(|t| t.z_pixels(w)).unwrap_or(0.)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use oc_mod::{
        illustration::IndexedIllustrations,
        nature::{NatureIndex, Traversability},
    };
    use oc_root::DEFAULT_FIGHT_DURATION;

    use super::*;

    // TODO: this test is not very good ... To make precise things, we must drop constant usage
    #[test]
    fn test_region_tiles() {
        // Given
        let w = WorldConfig::new(1000, 1000);
        let mod_ = Mod::new(
            "MyMod".to_string(),
            1,
            "A".to_string(),
            "B".to_string(),
            [0, 0, 0],
            [0, 0, 0],
            vec![],
            Default::default(),
            vec![],
            Default::default(),
            Default::default(),
            vec![],
            vec![],
            vec![],
            IndexedIllustrations::default(),
            vec![],
            vec![],
            Default::default(),
            vec![],
            1.5,
        );
        let meta = Meta::new("MyWorld".to_string(), 0, w.geo_meters_per_z().0, None);
        let tiles: Vec<Tile> = (0..w.tiles_count())
            .map(|i| Tile::new(WorldTileIndex(i), NatureIndex(0), 0, Traversability::all()))
            .collect();
        let altered_tiles = vec![None; tiles.len()];
        let decor = vec![None; tiles.len()];
        let world = World::new(
            w.clone(),
            mod_,
            meta,
            BattlePhase::Fight,
            DEFAULT_FIGHT_DURATION,
            Some(Instant::now()),
            BySide::default(),
            vec![],
            Flags::default(),
            Supports::default(),
            vec![],
            FxHashMap::default(),
            tiles,
            altered_tiles,
            decor,
            vec![],
            vec![],
            vec![Percent::ZERO; w.tiles_count() as usize],
            vec![],
            polyanya::Mesh::default(),
            Visibilities::empty(0),
            vec![],
            vec![],
            vec![],
            HashMap::default(),
            vec![],
            vec![],
            Uuids::default(),
        );

        // When
        let tiles = world.region_tiles(WorldRegionIndex(0));
        let tiles: Vec<(WorldTileIndex, Tile)> =
            tiles.into_iter().map(|(i, t)| (i, t.clone())).collect();

        // Then
        let mut expected = vec![];
        for y in 0..w.region_height() as usize {
            for x in 0..w.region_width() as usize {
                let i: WorldTileIndex = TileXy(Xy(x as u64, y as u64)).into_(&w);
                expected.push((i, Tile::new(i, NatureIndex(0), 0, Traversability::all())));
            }
        }
        assert_eq!(tiles, expected);
    }

    /// Warning: AI generated
    #[test]
    fn test_alter_tiles_changes_walls() {
        // Given
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mods/tests1");
        let mod_ = Mod::load(&root, None).unwrap();
        let grass = mod_.nature_index("ShortGrass").unwrap();
        let rock = mod_.nature_index("BigRock").unwrap();
        let w = WorldConfig::new(10, 10).with_geo_pixels_per_tile(5);
        let meta = Meta::new("MyWorld".to_string(), 0, w.geo_meters_per_z().0, None);
        let allow = mod_.nature(grass).traversability.clone();
        let tiles: Vec<Tile> = (0..w.tiles_count())
            .map(|i| Tile::new(WorldTileIndex(i), grass, 0, allow.clone()))
            .collect();
        let altered_tiles = vec![None; tiles.len()];
        let decor = vec![None; tiles.len()];
        let navmesh = navmesh::navmesh(&w, &tiles.as_walls(&mod_));
        let mut world = World::new(
            w.clone(),
            mod_,
            meta,
            BattlePhase::Fight,
            DEFAULT_FIGHT_DURATION,
            Some(Instant::now()),
            BySide::default(),
            vec![],
            Flags::default(),
            Supports::default(),
            vec![],
            FxHashMap::default(),
            tiles,
            altered_tiles,
            decor,
            vec![],
            vec![],
            vec![Percent::ZERO; w.tiles_count() as usize],
            vec![],
            navmesh,
            Visibilities::empty(0),
            vec![],
            vec![],
            vec![],
            HashMap::default(),
            vec![],
            vec![],
            Uuids::default(),
        );
        let wall = WorldTileIndex(0);
        let other = WorldTileIndex(w.tiles_count() - 1);
        let center = |i: WorldTileIndex| {
            let xy: TileXy = i.into_(&w);
            let p = xy.point(&w);
            let half = w.geo_pixels_per_tile() as f32 / 2.;
            (p[0] + half, p[1] + half)
        };
        assert!(world.navmesh.point_in_mesh(center(wall)));
        assert!(world.navmesh.point_in_mesh(center(other)));

        // When
        world.alter_tiles(vec![AlteredTile::new(wall, rock)]);

        // Then
        assert_eq!(world.tiles.at(wall).nature, rock);
        assert!(world.tiles.at(wall).allow.deny(MaterialKind::Individual));
        assert_eq!(
            *world.altered_tiles.at(wall),
            Some(AlteredTile::new(wall, rock))
        );
        assert!(!world.navmesh.point_in_mesh(center(wall)));
        assert!(world.navmesh.point_in_mesh(center(other)));
        assert_eq!(*world.altered_tiles.at(other), None);
    }
}
