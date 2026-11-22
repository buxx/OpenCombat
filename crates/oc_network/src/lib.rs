// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

use derive_more::Constructor;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_individual::{
    IndividualIndex,
    network::{Individual, Squad},
    order::{Order, OrderIndex},
    squad::SquadIndex,
};
use oc_mod::Mod;
use oc_physics::fx::Fx;
use oc_projectile::network::Projectile;
use oc_projectile::spawn::SpawnProjectiles;
use oc_root::{
    WorldConfig,
    battle::BattlePhase,
    geo::{WorldVec2, WorldVec3},
    identity::Identity,
    static_::StaticSource,
};
use oc_utils::percent::Percent;
use oc_vehicle::VehicleIndex;
use oc_world::{
    crater::Crater,
    decor::Decor,
    flag::Flags,
    meta::Meta,
    morale::Morale,
    resume::{EndResume, WorldResume},
    support::SupportType,
    tile::{AlteredTile, Tile},
    visibility::Visibility,
};

#[oc_derive::archivable(no_compare)]
#[derive(Debug, Clone)]
pub enum ToClient {
    GameConfig(Box<GameConfig>),
    WorldResume(WorldResume),
    EndResume(EndResume),
    Individual(Individual),
    Squad(Squad),
    Vehicle(oc_vehicle::network::Vehicle),
    Projectile(Projectile),
    Tiles(WorldRegionIndex, Vec<(WorldTileIndex, Tile)>),
    AlteredTiles(WorldRegionIndex, Vec<AlteredTile>),
    Decor(WorldRegionIndex, Vec<Decor>),
    RemovedDecor(WorldRegionIndex, Vec<WorldTileIndex>),
    Craters(WorldRegionIndex, Vec<Crater>),
    Smoke(WorldRegionIndex, Vec<(WorldTileIndex, Percent)>),
    Fx(Fx),
    /// Side individuals visibilities on opposite individuals, and on opposite vehicles
    UpdateVisibilities(
        Vec<(IndividualIndex, IndividualIndex, Visibility)>,
        Vec<(IndividualIndex, VehicleIndex, Visibility)>,
    ),
    SetBattlePhase(BattlePhase),
    Elapsed(std::time::Duration, std::time::Duration), // elapsed, total duration
    Flags(Flags),
    Morale(Morale),
    SetSupportCount(SupportType, u8),
    SupportExecuted(SupportType),
    Collision(WorldVec3, std::time::Duration), // Duration is lag
}

impl From<Individual> for ToClient {
    fn from(value: Individual) -> Self {
        ToClient::Individual(value)
    }
}

impl From<oc_vehicle::network::Vehicle> for ToClient {
    fn from(value: oc_vehicle::network::Vehicle) -> Self {
        ToClient::Vehicle(value)
    }
}

impl From<Projectile> for ToClient {
    fn from(value: Projectile) -> Self {
        ToClient::Projectile(value)
    }
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum ToServer {
    RequestInit(Identity),
    ListenRegion(WorldRegionIndex),
    ForgotRegion(WorldRegionIndex),
    Refresh,
    ExplodeProjectile(SpawnProjectiles),
    Squad(SquadIndex, SquadMessage),
    Fight,
    RequestSupport(SupportType, WorldVec2),
    #[cfg(feature = "debug")]
    DebugExplosion(oc_mod::shell::ShellIndex, WorldVec2),
    #[cfg(feature = "debug")]
    DebugRequestSupport(oc_root::side::Side, SupportType, WorldVec2),
    #[cfg(feature = "debug")]
    DebugEndBattle,
    #[cfg(feature = "debug")]
    DebugKillSquad(oc_root::side::Side, u32),
    /// Give ownership of all flags to given side
    #[cfg(feature = "debug")]
    DebugOwnFlags(oc_root::side::Side),
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct GameConfig {
    pub w: WorldConfig,
    pub mod_: Mod,
    pub meta: Meta,
    pub static_: StaticSource,
    pub persist: bool,
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum SquadMessage {
    SetOrders(Vec<Order>),
    SetPositionOrderPosition(OrderIndex, WorldVec2),
    SetLeaderPosition(WorldVec2),
}
