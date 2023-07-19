use oc_core::spawn::SpawnZoneName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapControl {
    spawn_zone_names: Vec<SpawnZoneName>,
}

impl MapControl {
    pub fn new(spawn_zone_names: Vec<SpawnZoneName>) -> Self {
        Self { spawn_zone_names }
    }

    pub fn empty() -> Self {
        Self {
            spawn_zone_names: vec![],
        }
    }

    pub fn contains_spawn_zone(&self, spawn_zone_name: &SpawnZoneName) -> bool {
        self.spawn_zone_names.contains(spawn_zone_name)
    }

    pub fn spawn_zone_names(&self) -> &Vec<SpawnZoneName> {
        self.spawn_zone_names.as_ref()
    }
}
