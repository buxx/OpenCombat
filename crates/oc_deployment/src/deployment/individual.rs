use oc_root::uuids::IndividualUuid;
use serde::{Deserialize, Serialize};

use crate::deployment::{magazine::Magazine, weapon::Weapons};

#[derive(Debug, Deserialize, Serialize)]
pub struct Individual {
    pub uuid: IndividualUuid,
    pub illustration: String,
    #[serde(default = "default_xp")]
    pub xp: u8,
    pub weapons: Weapons,
    pub magazines: Vec<Magazine>,
    #[serde(default)]
    pub status: Status,
}

/// Individual status kept between battles
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Status {
    #[default]
    Operational,
    Dead,
}

impl From<&oc_individual::Status> for Status {
    fn from(value: &oc_individual::Status) -> Self {
        match value {
            oc_individual::Status::Operational | oc_individual::Status::Stunned(_) => {
                Status::Operational
            }
            oc_individual::Status::Dead => Status::Dead,
        }
    }
}

impl Default for Individual {
    fn default() -> Self {
        Self::new()
    }
}

impl Individual {
    pub fn new() -> Self {
        Self {
            uuid: IndividualUuid::new_v4(),
            illustration: "default".to_string(),
            xp: default_xp(),
            weapons: Weapons::empty(),
            magazines: vec![],
            status: Status::default(),
        }
    }
}

fn default_xp() -> u8 {
    128
}
