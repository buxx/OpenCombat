use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_root::geo::WorldVec3;

/// Geographic consequence of a physic change, to be reflected by indexes.
#[derive(Debug, Clone)]
pub enum Effect {
    Position {
        before: WorldVec3,
        after: WorldVec3,
    },
    Tile {
        _before: WorldTileIndex,
        _after: WorldTileIndex,
    },
    Region {
        before: WorldRegionIndex,
        after: WorldRegionIndex,
    },
}
