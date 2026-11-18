use std::path::{Path, PathBuf};

use oc_world::check::check_map;

use crate::mods::{MODS_FOLDER, ModInfo};

const MAPS_FOLDER: &str = "maps";

#[derive(Debug, Clone)]
pub struct ModCompatibility {
    pub mod_name: String,
    pub errors: Vec<String>,
}

impl ModCompatibility {
    pub fn is_compatible(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    pub name: String,
    pub compatibility: Vec<ModCompatibility>,
}

pub fn load_all(mods: &[ModInfo]) -> (Vec<Map>, Vec<String>) {
    let mut maps = vec![];
    let mut errors = vec![];

    for path in map_folders(Path::new(MAPS_FOLDER), &mut errors) {
        match load(&path, mods) {
            Ok(map) => maps.push(map),
            Err(error) => errors.push(error),
        }
    }

    (maps, errors)
}

fn map_folders(folder: &Path, errors: &mut Vec<String>) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(format!("Read folder '{}': {error}", folder.display()));
            return vec![];
        }
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    paths.sort();
    paths
}

fn load(path: &Path, mods: &[ModInfo]) -> Result<Map, String> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid map folder name '{}'", path.display()))?
        .to_string();

    let compatibility = mods
        .iter()
        .map(|mod_| {
            let mod_path = Path::new(MODS_FOLDER).join(&mod_.name);
            let errors = check_map(&mod_path, path)
                .iter()
                .map(ToString::to_string)
                .collect();
            ModCompatibility {
                mod_name: mod_.name.clone(),
                errors,
            }
        })
        .collect();

    Ok(Map {
        name,
        compatibility,
    })
}
