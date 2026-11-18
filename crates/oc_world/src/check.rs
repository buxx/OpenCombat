use std::path::Path;

use oc_mod::Mod;

const TILE_ID_PROPERTY_KEY: &str = "ID";

#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("Unable to load mod: {0}")]
    Mod(String),
    #[error("Unable to load terrain tileset: {0}")]
    Terrain(String),
    #[error("Terrain tile {tile} ID '{id}' is not defined in mod natures")]
    UnknownNature { tile: u32, id: String },
}

pub fn check_map(mod_path: &Path, map_path: &Path) -> Vec<CheckError> {
    let mod_ = match Mod::load(&mod_path.to_path_buf(), None) {
        Ok(mod_) => mod_,
        Err(e) => return vec![CheckError::Mod(e.to_string())],
    };
    let tsx = map_path.join("terrain.tsx");
    let tileset = match tiled::Loader::new().load_tsx_tileset(&tsx) {
        Ok(tileset) => tileset,
        Err(e) => return vec![CheckError::Terrain(format!("{}: {e}", tsx.display()))],
    };

    tileset
        .tiles()
        .filter_map(
            |(tile, data)| match data.properties.get(TILE_ID_PROPERTY_KEY) {
                Some(tiled::PropertyValue::StringValue(id)) => mod_
                    .nature_index(id)
                    .is_none()
                    .then(|| CheckError::UnknownNature {
                        tile,
                        id: id.clone(),
                    }),
                _ => None, // Tile without ID is not a nature
            },
        )
        .collect()
}
