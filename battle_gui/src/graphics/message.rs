use battle_core::{game::explosive::ExplosiveType, types::WorldPoint};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GraphicsMessage {
    PushExplosionAnimation(WorldPoint, ExplosiveType),
    RemoveExplosionAnimation(WorldPoint),
    RecomputeDebugTerrainOpacity,
    ReloadSoldiersAsset,
    ReloadVehiclesAsset,
    ReloadExplosionsAsset,
    ReloadUiAsset,
    ReloadAll,
}
