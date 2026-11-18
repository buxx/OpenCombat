use std::{ops::Deref, path::PathBuf};

use oc_mod::{Mod, nature::NatureIndex};
use oc_root::WorldConfig;
use rustc_hash::FxHashMap;
use tiled::TileId;

#[derive(Debug)]
pub struct Terrain {
    pub w: WorldConfig,
    pub raw: tiled::Tileset,
    pub natures: FxHashMap<NatureIndex, TileId>,
}

impl Terrain {
    pub fn load(path: &PathBuf, w: WorldConfig, mod_: &Mod) -> Result<Self, Error> {
        let raw = tiled::Loader::new().load_tsx_tileset(path)?;
        let natures = mod_
            .natures
            .iter()
            .map(|n| (n.index(), n.name.clone()))
            .collect::<Vec<(NatureIndex, String)>>();
        let natures = oc_utils::tileset::extract(natures, &raw);
        let natures = natures.map_err(|e| Error::ExtractTiles(path.clone(), e))?;
        let natures = natures.into_iter().collect();
        Ok(Self { w, raw, natures })
    }

    pub fn columns(&self) -> u32 {
        self.raw.columns
    }

    pub fn rows(&self) -> u32 {
        self.tilecount / self.columns()
    }
}

impl Deref for Terrain {
    type Target = tiled::Tileset;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error during load of {0}")]
    Load(#[from] tiled::Error),
    #[error("Error during extract tiles from {0}: {1}")]
    ExtractTiles(PathBuf, String),
}
