use std::path::Path;

use oc_world::{reader::MapReader, spawn::SpawnZoneName};

use crate::Battle;

#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("Unable to load battle: {0}")]
    Battle(String),
    #[error("Unable to load map: {0}")]
    Map(String),
    #[error("Scenario '{scenario}': {side} spawn zone '{zone}' does not exist in map")]
    UnknownSpawnZone {
        scenario: String,
        side: char,
        zone: String,
    },
}

pub fn check_battle(battle_path: &Path, map_path: &Path) -> Vec<CheckError> {
    let battle = match Battle::from_ron_file(battle_path) {
        Ok(battle) => battle,
        Err(e) => return vec![CheckError::Battle(e.to_string())],
    };
    let spawn_zones = match MapReader::new(map_path).and_then(|map| map.spawn_zones()) {
        Ok(spawn_zones) => spawn_zones,
        Err(e) => return vec![CheckError::Map(e.to_string())],
    };

    let unknown = |scenario: &str, side: char, names: &[SpawnZoneName]| {
        names
            .iter()
            .filter(|name| !spawn_zones.iter().any(|zone| zone.name == **name))
            .map(|name| CheckError::UnknownSpawnZone {
                scenario: scenario.to_string(),
                side,
                zone: name.0.clone(),
            })
            .collect::<Vec<_>>()
    };

    battle
        .scenarios()
        .iter()
        .flat_map(|scenario| {
            scenario
                .sides()
                .iter()
                .flat_map(|(side, scenario_side)| {
                    unknown(
                        scenario.name(),
                        side.letter(),
                        &scenario_side.start_spawn_zones,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
