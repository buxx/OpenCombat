use oc_root::uuids::SquadUuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Squad {
    pub uuid: SquadUuid,
    pub label: String,
    pub illustration: String,
    pub individuals: Vec<super::individual::Individual>,
}

impl Default for Squad {
    fn default() -> Self {
        Self::new()
    }
}

impl Squad {
    pub fn new() -> Self {
        Self {
            uuid: SquadUuid::new_v4(),
            label: "default".to_string(),
            illustration: "".to_string(),
            individuals: vec![],
        }
    }
}
