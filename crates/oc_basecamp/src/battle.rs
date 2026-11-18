use std::path::Path;

use oc_battle::check::check_battle;
pub use oc_battle::{Battle, Scenario};

pub const BATTLES_FOLDER: &str = "battles";
pub const MAPS_FOLDER: &str = "maps";

pub fn load_all() -> (Vec<Battle>, Vec<String>) {
    let mut battles = vec![];
    let mut errors = vec![];

    let entries = match std::fs::read_dir(BATTLES_FOLDER) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(format!("Read folder '{BATTLES_FOLDER}': {error}"));
            return (battles, errors);
        }
    };

    let mut folders: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    folders.sort();

    for folder in folders {
        let Some(name) = folder.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let path = folder.join(format!("{name}.ron"));
        match Battle::from_ron_file(&path) {
            Ok(battle) => {
                let map_path = Path::new(MAPS_FOLDER).join(battle.map());
                errors.extend(
                    check_battle(&path, &map_path)
                        .iter()
                        .map(|e| format!("Battle '{}': {e}", battle.name())),
                );
                battles.push(battle);
            }
            Err(error) => errors.push(error.to_string()),
        }
    }

    (battles, errors)
}
