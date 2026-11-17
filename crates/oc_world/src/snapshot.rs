use oc_utils::percent::Percent;
use std::time::{Duration, Instant, SystemTime};

use derive_more::Constructor;
use oc_individual::squad::Squad;
use oc_individual::{Individual, Status, Weapons};
use oc_projectile::Projectile;
use oc_root::WorldConfig;
use oc_root::battle::BattlePhase;
use oc_root::geo::WorldVec2;
use oc_root::side::{BySide, Side};
use oc_root::uuids::{IndividualUuid, Uuids};
use rkyv::with::{AsUnixTime, Map};

use crate::World;
use crate::crater::Crater;
use crate::decor::Decor;
use crate::flag::Flags;
use crate::smoke::SmokeCloud;
use crate::spawn::{SpawnZoneName, owned_spawn_zones};
use crate::support::{PendingSupport, SupportType, Supports};
use crate::tile::{AlteredTile, Tile};
use oc_geo::tile::WorldTileIndex;
use oc_root::DEFAULT_FIGHT_DURATION;

#[oc_derive::archivable(bare)]
#[derive(Debug, Constructor)]
#[allow(clippy::too_many_arguments)]
pub struct Snapshot {
    pub w: WorldConfig,
    pub tiles: Vec<Tile>,
    pub altered_tiles: Vec<Option<AlteredTile>>,
    pub decor: Vec<Option<Decor>>,
    pub craters: Vec<Crater>,
    pub smoke: Vec<Percent>,
    pub smoke_clouds: Vec<SnapshotSmokeCloud>,
    pub individuals: Vec<Individual>,
    pub squads: Vec<Squad>,
    pub projectiles: Vec<Projectile>,
    pub phase: BattlePhase,
    pub duration: Duration,
    #[rkyv(with = Map<AsUnixTime>)]
    pub start: Option<SystemTime>,
    pub spawns: BySide<Vec<SpawnZoneName>>,
    pub supports: Supports,
    pub pending_supports: Vec<SnapshotPendingSupport>,
    pub support_realized: Vec<SnapshotSupportRealized>,
    pub uuids: Uuids,
}

/// Persistable form of a support request time (Instant can't be persisted)
#[oc_derive::archivable(bare)]
#[derive(Debug, Clone)]
pub struct SnapshotSupportRealized {
    pub side: Side,
    pub type_: SupportType,
    #[rkyv(with = AsUnixTime)]
    pub at: SystemTime,
}

impl SnapshotSupportRealized {
    pub fn new(side: Side, type_: SupportType, at: Instant) -> Self {
        Self {
            side,
            type_,
            at: SystemTime::now()
                .checked_sub(at.elapsed())
                .unwrap_or_else(SystemTime::now),
        }
    }
}

#[oc_derive::archivable(bare)]
#[derive(Debug, Clone)]
pub struct SnapshotSmokeCloud {
    #[rkyv(with = AsUnixTime)]
    pub start: SystemTime,
    pub hold: Duration,
    pub fade: Duration,
    pub tiles: Vec<(WorldTileIndex, Percent)>,
}

impl From<&SmokeCloud> for SnapshotSmokeCloud {
    fn from(value: &SmokeCloud) -> Self {
        Self {
            start: SystemTime::now()
                .checked_sub(value.start.elapsed())
                .unwrap_or_else(SystemTime::now),
            hold: value.hold,
            fade: value.fade,
            tiles: value.tiles.clone(),
        }
    }
}

impl From<SnapshotSmokeCloud> for SmokeCloud {
    fn from(value: SnapshotSmokeCloud) -> Self {
        let elapsed = value.start.elapsed().unwrap_or_default();
        let start = Instant::now()
            .checked_sub(elapsed)
            .unwrap_or_else(Instant::now);
        Self::new(start, value.hold, value.fade, value.tiles)
    }
}

/// Persistable form of PendingSupport (Instant can't be persisted)
#[oc_derive::archivable(bare)]
#[derive(Debug, Clone)]
pub struct SnapshotPendingSupport {
    pub type_: SupportType,
    pub side: Side,
    pub position: WorldVec2,
    #[rkyv(with = Map<AsUnixTime>)]
    pub start: Option<SystemTime>,
    pub delay: Duration,
}

impl From<&PendingSupport> for SnapshotPendingSupport {
    fn from(value: &PendingSupport) -> Self {
        Self {
            type_: value.type_.clone(),
            side: value.side,
            position: value.position,
            delay: value.delay,
            start: value.start.map(|start| {
                SystemTime::now()
                    .checked_sub(start.elapsed())
                    .unwrap_or_else(SystemTime::now)
            }),
        }
    }
}

impl From<SnapshotPendingSupport> for PendingSupport {
    fn from(value: SnapshotPendingSupport) -> Self {
        let start = value.start.map(|start| {
            let elapsed = start.elapsed().unwrap_or_default();
            Instant::now()
                .checked_sub(elapsed)
                .unwrap_or_else(Instant::now)
        });
        Self {
            type_: value.type_,
            side: value.side,
            position: value.position,
            start,
            delay: value.delay,
        }
    }
}

impl Snapshot {
    pub fn empty(w: WorldConfig) -> Self {
        Self {
            w,
            tiles: vec![],
            altered_tiles: vec![],
            decor: vec![],
            craters: vec![],
            smoke: vec![],
            smoke_clouds: vec![],
            individuals: vec![],
            squads: vec![],
            projectiles: vec![],
            phase: BattlePhase::Deployment,
            duration: DEFAULT_FIGHT_DURATION,
            start: None,
            spawns: BySide::default(),
            supports: Supports::default(),
            pending_supports: vec![],
            support_realized: vec![],
            uuids: Uuids::default(),
        }
    }

    pub fn with_tiles(mut self, tiles: Vec<Tile>) -> Self {
        self.altered_tiles = vec![None; tiles.len()];
        self.decor = vec![None; tiles.len()];
        self.smoke = vec![Percent::ZERO; tiles.len()];
        self.tiles = tiles;
        self
    }

    pub fn with_decor(mut self, decor: Vec<Option<Decor>>) -> Self {
        self.decor = decor;
        self
    }

    pub fn with_individuals(mut self, individuals: Vec<Individual>) -> Self {
        self.individuals = individuals;
        self
    }

    pub fn with_squads(mut self, squads: Vec<Squad>) -> Self {
        self.squads = squads;
        self
    }

    pub fn with_projectiles(mut self, projectiles: Vec<Projectile>) -> Self {
        self.projectiles = projectiles;
        self
    }

    pub fn with_phase(mut self, value: BattlePhase) -> Self {
        self.phase = value;
        self
    }

    pub fn with_supports(mut self, value: Supports) -> Self {
        self.supports = value;
        self
    }
}

#[oc_derive::archivable(bare)]
#[derive(Debug, Constructor)]
#[allow(clippy::too_many_arguments)]
pub struct State {
    pub altered_tiles: Vec<Option<AlteredTile>>,
    pub decor: Vec<Option<Decor>>,
    pub craters: Vec<Crater>,
    pub spawns: BySide<Vec<SpawnZoneName>>,
    pub individuals: Vec<IndividualState>,
    pub flags: Flags,
}

/// Individual state kept between battles
#[oc_derive::archivable(bare)]
#[derive(Debug, Clone, Constructor)]
pub struct IndividualState {
    pub uuid: IndividualUuid,
    pub status: Status,
    pub weapons: Weapons,
}

impl From<&World> for State {
    fn from(world: &World) -> Self {
        let spawns = |side| {
            owned_spawn_zones(
                &world.spawn_zones,
                world.spawn_zones(side),
                &world.flags,
                side,
            )
        };

        Self {
            altered_tiles: world.altered_tiles.clone(),
            decor: world.decor.clone(),
            craters: world.craters.clone(),
            spawns: BySide::from_fn(spawns),
            individuals: individuals_states(world),
            flags: world.flags.clone(),
        }
    }
}

fn individuals_states(world: &World) -> Vec<IndividualState> {
    world
        .individuals
        .iter()
        .enumerate()
        .map(|(i, individual)| {
            IndividualState::new(
                *world.uuids.individual(i),
                individual.status,
                individual.weapons.clone(),
            )
        })
        .collect()
}
