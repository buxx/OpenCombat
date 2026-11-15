use std::path::Path;

use bon::Builder;
use oc_deployment::{deployment::place::OrderPolicy, generator::profile::Profile};
use oc_root::side::{BySide, Side};
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
    sides: BySide<ScenarioSide>,
}

/// Scenario configuration of one side
#[derive(Debug, Builder)]
pub struct ScenarioSide {
    pub deploy: Profile,
    pub start_spawn_zones: Vec<SpawnZoneName>,
    pub squad_count: usize,
    pub default: Vec<String>,
    #[builder(default)]
    pub supports: Support,
    #[builder(default)]
    pub places: Vec<SquadPlace>,
    pub order: Option<OrderPolicy>,
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
    /// Side A squads initial order, like `Defend` (default: decided by battle gui)
    #[serde(default)]
    a_order: Option<OrderPolicy>,
    /// Side B squads initial order, like `Hide` (default: decided by battle gui)
    #[serde(default)]
    b_order: Option<OrderPolicy>,
}

impl Scenario {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn sides(&self) -> &BySide<ScenarioSide> {
        &self.sides
    }

    pub fn side(&self, side: Side) -> &ScenarioSide {
        &self.sides[side]
    }

    pub fn from_ron_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let dir = path.parent().unwrap_or(Path::new("."));
        let raw: ScenarioRaw = read_ron(path)?;

        Ok(Self {
            name: raw.name,
            description: raw.description,
            sides: BySide::new(
                ScenarioSide {
                    deploy: read_ron(&dir.join(raw.deploy_a))?,
                    start_spawn_zones: raw.a_start_spawn_zones,
                    squad_count: raw.a_squad_count,
                    default: raw.a_default,
                    supports: Support(raw.a_supports),
                    places: raw.a_places,
                    order: raw.a_order,
                },
                ScenarioSide {
                    deploy: read_ron(&dir.join(raw.deploy_b))?,
                    start_spawn_zones: raw.b_start_spawn_zones,
                    squad_count: raw.b_squad_count,
                    default: raw.b_default,
                    supports: Support(raw.b_supports),
                    places: raw.b_places,
                    order: raw.b_order,
                },
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_without_some() {
        let dir = std::env::temp_dir().join(format!("oc_battle_scenario_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("orders.ron");
        std::fs::write(&path, "#![enable(implicit_some)]\n(a_order: Hide, b_order: Some(Defend))").unwrap();

        #[derive(Deserialize)]
        struct Orders {
            a_order: Option<OrderPolicy>,
            b_order: Option<OrderPolicy>,
        }
        let orders: Orders = read_ron(&path).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(orders.a_order, Some(OrderPolicy::Hide));
        assert_eq!(orders.b_order, Some(OrderPolicy::Defend));
    }
}
