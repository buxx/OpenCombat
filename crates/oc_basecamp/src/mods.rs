use std::path::PathBuf;

use serde::Deserialize;

pub const MODS_FOLDER: &str = "mods";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "Mod")]
pub struct ModInfo {
    pub name: String,
    pub a_name: String,
    pub b_name: String,
    pub a_color: [u8; 3],
    pub b_color: [u8; 3],
}

pub fn list() -> (Vec<ModInfo>, Vec<String>) {
    let mut folders: Vec<PathBuf> = std::fs::read_dir(MODS_FOLDER)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect()
        })
        .unwrap_or_default();
    folders.sort();

    let mut mods = vec![];
    let mut errors = vec![];
    for folder in folders {
        let path = folder.join("mod.ron");
        match oc_root::format::read_ron(&path) {
            Ok(info) => mods.push(info),
            Err(error) => errors.push(format!("Read mod: {error}")),
        }
    }

    (mods, errors)
}
