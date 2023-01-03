use serde::{Deserialize, Serialize};
use std::cmp::min;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Feeling {
    UnderFire(u32),
}

const UNDER_FIRE_TICK: u32 = 10;
const UNDER_FIRE_MAX: u32 = 200;
const UNDER_FIRE_DANGER: u32 = 150;
const UNDER_FIRE_WARNING: u32 = 100;

impl Feeling {
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

    pub fn value(&self) -> u32 {
        match self {
            Feeling::UnderFire(value) => *value,
        }
    }
}
