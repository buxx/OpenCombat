use std::path::Path;

use bon::Builder;
use oc_deployment::generator::profile::Profile;
use oc_world::{
    spawn::SpawnZoneName,
    support::{Support, SupportType},
};
use serde::Deserialize;

use crate::{Error, read_ron};

#[derive(Debug, Builder)]
pub struct Scenario {
    name: String,
    description: String,
    deploy_a: Profile,
    deploy_b: Profile,
    a_start_spawn_zones: Vec<SpawnZoneName>,
    b_start_spawn_zones: Vec<SpawnZoneName>,
    a_squad_count: usize,
    b_squad_count: usize,
    a_default: Vec<String>,
    b_default: Vec<String>,
    #[builder(default)]
    a_supports: Support,
    #[builder(default)]
    b_supports: Support,
    #[builder(default)]
    a_places: Vec<SquadPlace>,
    #[builder(default)]
    b_places: Vec<SquadPlace>,
}

/// Squad (by its label, like `"Mg34"`) to place at a map place (by its name, like `"50"`)
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SquadPlace(pub String, pub String);

impl SquadPlace {
    pub fn squad(&self) -> &str {
        &self.0
    }

    pub fn place(&self) -> &str {
        &self.1
    }
}

#[derive(Deserialize)]
struct ScenarioRaw {
    name: String,
    description: String,
    deploy_a: String,
    deploy_b: String,
    a_start_spawn_zones: Vec<SpawnZoneName>,
    b_start_spawn_zones: Vec<SpawnZoneName>,
    a_squad_count: usize,
    b_squad_count: usize,
    a_default: Vec<String>,
    b_default: Vec<String>,
    /// Side A available supports, like `[(NavalBombard, 3)]`
    #[serde(default)]
    a_supports: Vec<(SupportType, u8)>,
    /// Side B available supports, like `[(NavalBombard, 3)]`
    #[serde(default)]
    b_supports: Vec<(SupportType, u8)>,
    /// Side A squads to place at map places, like `[("Riffle", "63")]`
    #[serde(default)]
    a_places: Vec<SquadPlace>,
    /// Side B squads to place at map places, like `[("Mg34", "50")]`
    #[serde(default)]
    b_places: Vec<SquadPlace>,
}

impl Scenario {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn deploy_a(&self) -> &Profile {
        &self.deploy_a
    }

    pub fn deploy_b(&self) -> &Profile {
        &self.deploy_b
    }

    pub fn a_start_spawn_zones(&self) -> &[SpawnZoneName] {
        &self.a_start_spawn_zones
    }

    pub fn b_start_spawn_zones(&self) -> &[SpawnZoneName] {
        &self.b_start_spawn_zones
    }

    pub fn a_squad_count(&self) -> usize {
        self.a_squad_count
    }

    pub fn b_squad_count(&self) -> usize {
        self.b_squad_count
    }

    pub fn a_default(&self) -> &[String] {
        &self.a_default
    }

    pub fn b_default(&self) -> &[String] {
        &self.b_default
    }

    pub fn a_supports(&self) -> &Support {
        &self.a_supports
    }

    pub fn b_supports(&self) -> &Support {
        &self.b_supports
    }

    pub fn a_places(&self) -> &[SquadPlace] {
        &self.a_places
    }

    pub fn b_places(&self) -> &[SquadPlace] {
        &self.b_places
    }

    pub fn from_ron_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let dir = path.parent().unwrap_or(Path::new("."));
        let raw: ScenarioRaw = read_ron(path)?;

        Ok(Self {
            name: raw.name,
            description: raw.description,
            deploy_a: read_ron(&dir.join(raw.deploy_a))?,
            deploy_b: read_ron(&dir.join(raw.deploy_b))?,
            a_start_spawn_zones: raw.a_start_spawn_zones,
            b_start_spawn_zones: raw.b_start_spawn_zones,
            a_squad_count: raw.a_squad_count,
            b_squad_count: raw.b_squad_count,
            a_default: raw.a_default,
            b_default: raw.b_default,
            a_supports: Support(raw.a_supports),
            b_supports: Support(raw.b_supports),
            a_places: raw.a_places,
            b_places: raw.b_places,
        })
    }
}
