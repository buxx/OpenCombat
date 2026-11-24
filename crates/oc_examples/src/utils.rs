//! Helpers shared by examples
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use anyhow::Context;
use oc_deployment::deployment::{Deployments, place::OrderPolicy};
use oc_geo::tile::WorldTileIndex;
use oc_individual::{Individual, IndividualIndex, Weapons, order::Order, squad::Squad};
use oc_mod::{Mod, ammunition::AmmunitionIndex, armament::ShotModeIndex, weapons::WeaponIndex};
use oc_projectile::spawn::SpawnProjectiles;
use oc_root::{
    WcfgFrom, WorldConfig,
    format::RkyvFile,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    side::{BySide, Side},
    uuids::SquadUuid,
};
use oc_utils::index::AtIndex;
use oc_world::{
    load::WorldPath,
    meta::Meta,
    place::PlaceName,
    reader::MapReader,
    snapshot::Snapshot,
    spawn::SpawnZone,
    support::{Support, SupportType, Supports},
    tile::Tile,
};
use tests::{
    individual::TestIndividual,
    squad::TestSquad,
    weapons::{TestWeapon, TestWeapons},
};

/// Support declared by mods std1 and tests1
pub const NAVAL_BOMBARD: &str = "Naval bombard";

pub fn find_spawn(spawns: &[SpawnZone], name: &str) -> SpawnZone {
    spawns
        .iter()
        .find(|s| s.name.0 == name)
        .unwrap_or_else(|| panic!("Spawn zone '{name:?}' not found"))
        .clone()
}

/// Mod, map and its world config, loaded from mod and map directories
pub struct Loaded {
    pub mod_: Mod,
    pub map: MapReader,
    pub meta: Meta,
    pub w: WorldConfig,
}

impl Loaded {
    pub fn new(mod_: &Path, map: &Path) -> anyhow::Result<Self> {
        let mod_ =
            Mod::load(&mod_.to_path_buf(), None).context(format!("Load mod {}", mod_.display()))?;
        let (map, meta, w) = load_map(map)?;
        Ok(Self { mod_, map, meta, w })
    }

    pub fn tiles(&self, w: &WorldConfig) -> anyhow::Result<Vec<Tile>> {
        self.map.tiles(w, &self.mod_).context("Read map tiles")
    }
}

/// Map reader, meta and world config of map directory
pub fn load_map(map: &Path) -> anyhow::Result<(MapReader, Meta, WorldConfig)> {
    let meta_path = map.to_path_buf().meta();
    let meta = Meta::from_file(&meta_path).context(format!("Read {}", meta_path.display()))?;
    let reader = MapReader::new(map).context(format!("Read map {}", map.display()))?;
    let w = WorldConfig::new(
        reader.width().context("Read map width")? as u64,
        reader.height().context("Read map height")? as u64,
    )
    .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    Ok((reader, meta, w))
}

/// Save snapshot into a (kept) temporary file and return its path
pub fn save_snapshot(snapshot: &Snapshot) -> anyhow::Result<PathBuf> {
    let (_, path) = tempfile::NamedTempFile::new()?.keep()?;
    snapshot
        .save(&path)
        .context(format!("Save snapshot to {}", path.display()))?;
    Ok(path)
}

/// Position on the ground (z of the tile) at given position
pub fn on_ground(w: &WorldConfig, tiles: &[Tile], position: WorldVec2) -> WorldVec3 {
    let z = tiles.at(WorldTileIndex::from_(position, w)).z_pixels(w);
    position.extend(z)
}

/// Weapons with only given (filled) primary weapon
pub fn armed(mod_: &Mod, weapon: &str) -> Weapons {
    TestWeapons::builder()
        .primary(TestWeapon::filled(mod_, weapon).make())
        .build()
        .make()
}

/// Individual without weapon
pub fn soldier(w: &WorldConfig, side: Side, position: impl Into<WorldVec3>) -> Individual {
    TestIndividual::builder()
        .side(side)
        .position(position.into())
        .build()
        .make(w)
}

/// Individual with only given (filled) primary weapon
pub fn armed_soldier(
    w: &WorldConfig,
    mod_: &Mod,
    side: Side,
    position: impl Into<WorldVec3>,
    weapon: &str,
) -> Individual {
    TestIndividual::builder()
        .side(side)
        .position(position.into())
        .weapons(armed(mod_, weapon))
        .build()
        .make(w)
}

/// Squad of given members, positioned on the first one. Squad side is the [`TestSquad`] default
/// one (A), whatever members side.
pub fn squad(individuals: &[Individual], members: &[u64], orders: Vec<Order>) -> Squad {
    TestSquad::builder()
        .position(individuals[members[0] as usize].position.into())
        .members(members.iter().copied().map(IndividualIndex).collect())
        .orders(orders)
        .build()
        .make()
}

/// Weapon, ammunition and shot mode used to spawn projectiles
#[derive(Debug, Clone, Copy)]
pub struct Firing {
    pub weapon: WeaponIndex,
    pub ammunition: AmmunitionIndex,
    pub shot: ShotModeIndex,
}

impl Firing {
    /// Panic if not found in mod
    pub fn find(mod_: &Mod, weapon: &str, ammunition: &str, shot: &str) -> Self {
        let weapon_ = mod_
            .weapons
            .iter()
            .find(|w| w.name() == weapon)
            .unwrap_or_else(|| panic!("Weapon '{weapon}' not found"));
        let ammunition_ = weapon_
            .ammunitions()
            .iter()
            .find(|a| a.name() == ammunition)
            .unwrap_or_else(|| panic!("Ammunition '{ammunition}' not found"));
        let shot_ = weapon_
            .shots()
            .iter()
            .find(|s| s.name() == shot)
            .unwrap_or_else(|| panic!("Shot '{shot}' not found"));
        Self {
            weapon: weapon_.index(),
            ammunition: ammunition_.index(),
            shot: shot_.index(),
        }
    }

    /// "Weapon1" with "Ammo1" and "Single" shot (declared by tests1 mod)
    pub fn weapon1(mod_: &Mod) -> Self {
        Self::find(mod_, "Weapon1", "Ammo1", "Single")
    }

    pub fn spawn(
        &self,
        repeat: u8,
        from: impl Into<WorldVec3>,
        directions: Vec<WorldVec3>,
        side: Side,
    ) -> SpawnProjectiles {
        SpawnProjectiles::new(
            self.weapon,
            self.ammunition,
            self.shot,
            repeat,
            from.into(),
            directions,
            side,
            Duration::ZERO,
            None,
        )
    }
}

/// Normalized direction from `from` to `to`
pub fn direction(from: impl Into<WorldVec3>, to: impl Into<WorldVec3>) -> WorldVec3 {
    (to.into() - from.into()).normalize_or_zero()
}

/// Side A owns `count` naval bombard supports
pub fn naval_bombards(count: u8) -> Supports {
    Supports::new(
        Support(vec![(SupportType::new(NAVAL_BOMBARD), count)]),
        Support::default(),
    )
}

/// Map places by name
pub fn places(map: &MapReader) -> anyhow::Result<HashMap<PlaceName, WorldVec2>> {
    Ok(map
        .build()
        .context("Build map")?
        .places()
        .iter()
        .map(|p| (p.name.clone(), WorldVec2::new(p.x, p.y)))
        .collect())
}

/// Squad (by uuid) assigned to place (by name). Panic if not found.
pub fn squad_at(
    places: &HashMap<PlaceName, WorldVec2>,
    squad: &str,
    place: &str,
) -> (SquadUuid, WorldVec2) {
    (
        squad
            .parse::<SquadUuid>()
            .unwrap_or_else(|_| panic!("Invalid squad uuid '{squad}'")),
        *places
            .get(&place.into())
            .unwrap_or_else(|| panic!("Place '{place}' not found")),
    )
}

/// Played side hides, other side defends
pub fn hide_and_defend(side: Side) -> BySide<OrderPolicy> {
    match side {
        Side::A => BySide::new(OrderPolicy::Hide, OrderPolicy::Defend),
        Side::B => BySide::new(OrderPolicy::Defend, OrderPolicy::Hide),
    }
}

/// Snapshot of map tiles and decor, with supports and spawn zones of deployments
pub fn deployment_snapshot(
    w: &WorldConfig,
    map: &MapReader,
    tiles: Vec<Tile>,
    supports: Supports,
    deployments: &Deployments,
) -> anyhow::Result<Snapshot> {
    let mut snapshot = Snapshot::empty(w.clone())
        .with_tiles(tiles)
        .with_decor(map.decor().context("Read map decor")?)
        .with_supports(supports);
    snapshot.spawns = deployments
        .as_ref()
        .map(|d| d.spawns.iter().map(|s| s.name.clone()).collect());
    Ok(snapshot)
}

/// Panic if `--test` is given without `test` feature
pub fn ensure_test_feature(test: bool) {
    if test && !cfg!(feature = "test") {
        panic!("To enable test, feature `test` must be enabled too")
    }
}

/// Panic if `--test` is given without `test` and `debug` features
pub fn ensure_test_debug_features(test: bool) {
    if test && !(cfg!(feature = "test") && cfg!(feature = "debug")) {
        panic!("To enable test, feature `test` and `debug` must be enabled too")
    }
}

/// Error if test success flag is not set
pub fn test_result(success: &AtomicBool) -> anyhow::Result<()> {
    if !success.load(Ordering::Relaxed) {
        anyhow::bail!("❌ Test failed")
    }
    println!("✅ Test success !");
    Ok(())
}

/// Server tracker queries, to check expectations after app exit
#[cfg(feature = "test")]
pub mod tracked {
    use oc_individual::IndividualIndex;
    use oc_physics::Event;
    use oc_world_server::{state::ObjectId, tracker::Tracker_};

    /// Count of projectile collisions with objects matching `with`
    fn projectile_collisions(tracker: &Tracker_, with: impl Fn(&ObjectId) -> bool) -> usize {
        tracker
            .physics
            .iter()
            .filter(|event| {
                matches!(event, Event::Collision(ObjectId::Projectile(_), object, _) if with(object))
            })
            .count()
    }

    pub fn tile_collisions(tracker: &Tracker_) -> usize {
        projectile_collisions(tracker, |o| matches!(o, ObjectId::Tile(_)))
    }

    pub fn wreck_collisions(tracker: &Tracker_) -> usize {
        projectile_collisions(tracker, |o| matches!(o, ObjectId::Wreck(_)))
    }

    pub fn individual_collisions(tracker: &Tracker_) -> usize {
        projectile_collisions(tracker, |o| matches!(o, ObjectId::Individual(_)))
    }

    /// Individual received a dead status
    pub fn dead(tracker: &Tracker_, individual: IndividualIndex) -> bool {
        tracker.individuals.iter().any(|(i, update)| {
            *i == individual
                && matches!(
                    update,
                    oc_individual::Update::SetStatus(oc_individual::Status::Dead)
                )
        })
    }
}
