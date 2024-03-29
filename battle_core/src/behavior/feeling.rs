use serde::{Deserialize, Serialize};
use std::cmp::min;

use crate::types::Distance;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Feeling {
    UnderFire(u32),
}

pub const UNDER_FIRE_TICK: u32 = 10;
pub const UNDER_FIRE_MAX: u32 = 200;
pub const UNDER_FIRE_DANGER: u32 = 150;
pub const UNDER_FIRE_WARNING: u32 = 100;

impl Feeling {
    pub fn blast_increase_value(distance: Distance) -> u32 {
        if distance.meters() < 5 {
            150
        } else if distance.meters() < 10 {
            100
        } else {
            50
        }
    }

    pub fn proximity_bullet_increase_value(distance: Distance) -> u32 {
        if distance.meters() < 3 {
            100
        } else if distance.meters() < 10 {
            35
        } else {
            1
        }
    }

    pub fn decrease(&mut self) {
        match self {
            Feeling::UnderFire(value) => {
                if *value < UNDER_FIRE_TICK {
                    *value = 0;
                } else {
                    *value -= UNDER_FIRE_TICK
                }
            }
        }
    }

    pub fn increase(&mut self, add: u32) {
        match self {
            Feeling::UnderFire(value) => *value = min(*value + add, UNDER_FIRE_MAX),
        }
    }

    pub fn is_warning(&self) -> bool {
        match self {
            Feeling::UnderFire(value) => *value >= UNDER_FIRE_WARNING && *value < UNDER_FIRE_DANGER,
        }
    }

    pub fn is_danger(&self) -> bool {
        match self {
            Feeling::UnderFire(value) => *value >= UNDER_FIRE_DANGER && *value < UNDER_FIRE_MAX,
        }
    }

    pub fn is_max(&self) -> bool {
        match self {
            Feeling::UnderFire(value) => *value >= UNDER_FIRE_MAX,
        }
    }

    pub fn value_mut(&mut self) -> &mut u32 {
        match self {
            Feeling::UnderFire(value) => value,
        }
    }

    pub fn value(&self) -> &u32 {
        match self {
            Feeling::UnderFire(value) => value,
        }
    }

    pub fn exist(&self) -> bool {
        match self {
            Feeling::UnderFire(value) => *value > 0,
        }
    }
}
