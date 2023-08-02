use serde::{Deserialize, Serialize};

use crate::health::Health;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Morale(pub f32);

impl Morale {
    pub fn from_health(health: &Health) -> Self {
        Self(match health {
            Health::Good => 1.0,
            Health::Unconscious => 0.5,
            Health::Dead => 0.,
        })
    }
}
