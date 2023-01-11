use serde::{Deserialize, Serialize};

use crate::{
    game::explosive::Type as ExplosiveType,
    types::{Meters, SoldierIndex, VehicleIndex},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Effect {
    KillingBlast(SoldierIndex),
    StunningBlast(SoldierIndex),
    ProximityBlast(SoldierIndex, Meters),
    KillingBullet(SoldierIndex),
    InjuringBullet(SoldierIndex),
    ProximityBullet(SoldierIndex, Meters),
    VehicleShellImpact(VehicleIndex, ExplosiveType),
}
