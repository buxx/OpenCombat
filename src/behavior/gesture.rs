use serde::{Deserialize, Serialize};

use crate::{
    entity::soldier::WeaponClass,
    types::{Precision, SoldierIndex, WorldPoint},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Gesture {
    Idle,
    // Covering,
    Reloading(u64, WeaponClass),
    Aiming(u64, WeaponClass),
    Firing(u64, WeaponClass),
    // Firing,
    // ReloadingOwn,
    // ReloadingAsAssistant,
}
impl Gesture {
    pub fn next(&self, frame_i: u64, next: Gesture) -> Gesture {
        match self {
            Gesture::Idle => next,
            Gesture::Reloading(end, _) | Gesture::Aiming(end, _) | Gesture::Firing(end, _) => {
                if end <= &frame_i {
                    next
                } else {
                    self.clone()
                }
            }
        }
    }
}

pub enum GestureContext {
    Idle,
    Reloading,
    Aiming,
    Firing(WorldPoint, Option<(SoldierIndex, Precision)>),
}
