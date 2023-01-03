use serde::{Deserialize, Serialize};

use crate::types::{Meters, SoldierIndex};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Effect {
    KillingBlast(SoldierIndex),
    StunningBlast(SoldierIndex),
    ProximityBlast(SoldierIndex, Meters),
}
