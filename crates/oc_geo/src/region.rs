use oc_root::{WcfgFrom, WcfgInto, WorldConfig};
use oc_utils::d2::Xy;

use crate::tile::{TileXy, WorldTileIndex};

#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionXy(pub Xy);

impl From<RegionXy> for (u64, u64) {
    fn from(value: RegionXy) -> Self {
        (value.0.0, value.0.1)
    }
}

oc_utils::index_type! {
    pub struct WorldRegionIndex(pub u64);
}

impl WorldRegionIndex {
    pub fn background_file_name(&self) -> String {
        format!("{}.png", self.0)
    }
}

impl WcfgFrom<WorldRegionIndex> for RegionXy {
    fn from_(WorldRegionIndex(i): WorldRegionIndex, w: &WorldConfig) -> Self {
        let x = i % w.regions_width();
        let y = i / w.regions_width();
        Self(Xy(x, y))
    }
}

impl WcfgFrom<WorldRegionIndex> for Xy {
    fn from_(WorldRegionIndex(i): WorldRegionIndex, w: &WorldConfig) -> Self {
        let x = i % w.regions_width();
        let y = i / w.regions_width();
        Xy(x, y)
    }
}

impl WcfgFrom<RegionXy> for WorldRegionIndex {
    fn from_(RegionXy(Xy(x, y)): RegionXy, w: &WorldConfig) -> Self {
        Self(y * w.regions_width() + x)
    }
}

impl WcfgFrom<WorldTileIndex> for WorldRegionIndex {
    fn from_(tile: WorldTileIndex, w: &WorldConfig) -> Self {
        let tile_xy: TileXy = tile.into_(w);
        let region_xy: RegionXy = tile_xy.into_(w);
        region_xy.into_(w)
    }
}

impl WcfgFrom<TileXy> for RegionXy {
    fn from_(value: TileXy, w: &WorldConfig) -> Self {
        Self(Xy(
            value.0.0 / w.region_width(),
            value.0.1 / w.region_height(),
        ))
    }
}

impl WcfgFrom<TileXy> for WorldRegionIndex {
    fn from_(tile: TileXy, w: &WorldConfig) -> Self {
        let region_xy: RegionXy = tile.into_(w);
        region_xy.into_(w)
    }
}

pub trait Region {
    fn region(&self) -> WorldRegionIndex;
    fn set_region(&mut self, value: WorldRegionIndex);
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(9, 0, 9)]
    #[case(10, 0, 10)]
    #[case(9, 1, 19)]
    #[case(11, 1, 21)]
    pub fn test_world_region_index_from_world_region_xy(
        #[case] x: u64,
        #[case] y: u64,
        #[case] i: u64,
    ) {
        // Given

        let w = WorldConfig::new(1000, 1000)
            .with_region_width(100)
            .with_region_height(100);

        // When
        let index = WorldRegionIndex::from_(RegionXy(Xy(x, y)), &w);

        // Then
        assert_eq!(index.0, i);
    }

    #[rstest]
    #[case(0, 0, 0)] // Top left tile is de facto in first region
    #[case(100, 0, 1)] // First tile in second region
    #[case(0, 1, 0)] // First tile in second world row is first region of world
    #[case(0, 100, 10)]
    pub fn test_world_region_index_from_world_tile_index(
        #[case] x: u64,
        #[case] y: u64,
        #[case] i: u64,
    ) {
        // Given

        let w = WorldConfig::new(1000, 1000)
            .with_region_width(100)
            .with_region_height(100);

        // When
        let tile_index = WorldTileIndex::from_(TileXy(Xy(x, y)), &w);
        let region_index = WorldRegionIndex::from_(tile_index, &w);

        // Then
        assert_eq!(region_index.0, i);
    }
}
