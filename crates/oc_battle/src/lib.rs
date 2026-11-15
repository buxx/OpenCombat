use std::path::Path;

use bon::Builder;
use serde::Deserialize;

pub mod check;
pub mod scenario;

pub use scenario::{Scenario, SquadPlace};

#[derive(Debug, Builder)]
pub struct Battle {
    name: String,
    description: String,
    map: String,
    #[builder(default)]
    scenarios: Vec<Scenario>,
}

pub use oc_root::format::{FileError as Error, read_ron};

#[derive(Deserialize)]
struct BattleRaw {
    name: String,
    description: String,
    map: String,
    scenarios: Vec<String>,
}

impl Battle {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn map(&self) -> &str {
        &self.map
    }

    pub fn scenarios(&self) -> &[Scenario] {
        &self.scenarios
    }

    pub fn from_ron_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let dir = path.parent().unwrap_or(Path::new("."));
        let raw: BattleRaw = read_ron(path)?;

        let scenarios = raw
            .scenarios
            .iter()
            .map(|file| Scenario::from_ron_file(dir.join(file)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name: raw.name,
            description: raw.description,
            map: raw.map,
            scenarios,
        })
    }
}
