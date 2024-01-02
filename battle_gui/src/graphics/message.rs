use battle_core::{
    game::{explosive::ExplosiveType, weapon::WeaponSprite},
    graphics::soldier::SoldierAnimationType,
    types::{Angle, WorldPoint},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GraphicsMessage {
    PushExplosionAnimation(WorldPoint, ExplosiveType),
    PushCanonBlastAnimation(WorldPoint, Angle, WeaponSprite, SoldierAnimationType),
    RemoveExplosionAnimation(WorldPoint),
    RemoveCanonBlastAnimation(WorldPoint),
    RecomputeDebugTerrainOpacity,
    ReloadSoldiersAsset,
    ReloadVehiclesAsset,
    ReloadExplosionsAsset,
    ReloadUiAsset,
    ReloadAll,
}
