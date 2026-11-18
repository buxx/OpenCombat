// use crate::spawn::SpawnZone;
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize, Clone, Default)]
// pub struct MapControl {
//     spawns: Vec<SpawnZoneName>,
// }

// impl MapControl {
//     pub fn new(spawns: Vec<SpawnZoneName>) -> Self {
//         Self { spawns }
//     }

//     pub fn contains(&self, name: &SpawnZoneName) -> bool {
//         self.spawns.contains(name)
//     }

//     pub fn spawns(&self) -> &[SpawnZoneName] {
//         &self.spawns
//     }
// }
