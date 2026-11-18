use oc_geo::tile::WorldTileIndex;
use oc_mod::{Mod, nature::NatureIndex};
use oc_root::WorldConfig;
use oc_world::{reader::MapReader, tile::Tile};

pub trait TilesGenerator {
    fn tiles(&self, w: &WorldConfig, mod_: &Mod) -> Vec<Tile>;
}

pub struct SameTileFiller(pub NatureIndex);

impl TilesGenerator for SameTileFiller {
    fn tiles(&self, w: &WorldConfig, mod_: &Mod) -> Vec<Tile> {
        (0..w.tiles_count())
            .map(|i| {
                let prohibe = mod_.nature(self.0).traversability.clone();
                Tile::new(WorldTileIndex(i), self.0, 0, prohibe)
            })
            .collect()
    }
}

impl TilesGenerator for MapReader {
    fn tiles(&self, w: &WorldConfig, mod_: &Mod) -> Vec<Tile> {
        MapReader::tiles(self, w, mod_).unwrap() // TODO
    }
}
