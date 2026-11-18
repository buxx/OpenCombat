use oc_world::spawn::SpawnZone;

pub fn find_spawn(spawns: &[SpawnZone], name: &str) -> SpawnZone {
    spawns
        .iter()
        .find(|s| s.name.0 == name)
        .unwrap_or_else(|| panic!("Spawn zone '{name:?}' not found"))
        .clone()
}
