use oc_root::{
    WcfgFrom, WorldConfig,
    geo::{WorldVec2, WorldVec3},
};
use oc_utils::{d2::Xy, true_};

use crate::region::RegionXy;

#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileXy(pub Xy);

impl TileXy {
    pub fn clamped(&self, w: &WorldConfig) -> Self {
        Self(Xy(
            self.0.0.min(w.world_width() - 1),
            self.0.1.min(w.world_height() - 1),
        ))
    }

    pub fn point(&self, w: &WorldConfig) -> [f32; 2] {
        [
            self.0.0 as f32 * w.geo_pixels_per_tile() as f32,
            self.0.1 as f32 * w.geo_pixels_per_tile() as f32,
        ]
    }
}

impl From<TileXy> for (u64, u64) {
    fn from(value: TileXy) -> Self {
        (value.0.0, value.0.1)
    }
}

impl WcfgFrom<TileXy> for WorldVec2 {
    fn from_(value: TileXy, w: &WorldConfig) -> Self {
        WorldVec2::new(
            value.0.0 as f32 * w.geo_pixels_per_tile() as f32,
            value.0.1 as f32 * w.geo_pixels_per_tile() as f32,
        )
    }
}

impl WcfgFrom<WorldVec2> for TileXy {
    fn from_(value: WorldVec2, w: &WorldConfig) -> Self {
        TileXy::from_([value.x, value.y], w)
    }
}

impl WcfgFrom<TileXy> for WorldVec3 {
    fn from_(value: TileXy, w: &WorldConfig) -> Self {
        WorldVec3::new(
            value.0.0 as f32 * w.geo_pixels_per_tile() as f32,
            value.0.1 as f32 * w.geo_pixels_per_tile() as f32,
            0.,
        )
    }
}

impl WcfgFrom<WorldVec3> for TileXy {
    fn from_(value: WorldVec3, w: &WorldConfig) -> Self {
        TileXy::from_([value.x, value.y], w)
    }
}

// a little bit tricky ...
impl WcfgFrom<TileXy> for [f32; 2] {
    fn from_(value: TileXy, w: &WorldConfig) -> Self {
        [
            value.0.0 as f32 * w.geo_pixels_per_tile() as f32,
            value.0.1 as f32 * w.geo_pixels_per_tile() as f32,
        ]
    }
}

// a little bit tricky ...
impl WcfgFrom<(f32, f32)> for TileXy {
    fn from_(value: (f32, f32), w: &WorldConfig) -> Self {
        TileXy(Xy(
            value.0 as u64 / w.geo_pixels_per_tile(),
            value.1 as u64 / w.geo_pixels_per_tile(),
        ))
    }
}

// a little bit tricky ...
impl WcfgFrom<glam::Vec2> for TileXy {
    fn from_(value: glam::Vec2, w: &WorldConfig) -> Self {
        TileXy(Xy(
            value.x as u64 / w.geo_pixels_per_tile(),
            value.y as u64 / w.geo_pixels_per_tile(),
        ))
    }
}

// a little bit tricky ...
impl WcfgFrom<glam::Vec3> for TileXy {
    fn from_(value: glam::Vec3, w: &WorldConfig) -> Self {
        TileXy(Xy(
            value.x as u64 / w.geo_pixels_per_tile(),
            value.y as u64 / w.geo_pixels_per_tile(),
        ))
    }
}

// a little bit tricky ...
impl WcfgFrom<TileXy> for [f32; 3] {
    fn from_(value: TileXy, w: &WorldConfig) -> Self {
        [
            value.0.0 as f32 * w.geo_pixels_per_tile() as f32,
            value.0.1 as f32 * w.geo_pixels_per_tile() as f32,
            0.0,
        ]
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldTileIndex(pub u64);

impl WorldTileIndex {
    // WARNING: AI generated
    /// Deterministic pseudo-random number (splitmix64) from tile index and salt: always the same
    /// for a given tile
    pub fn seeded(&self, salt: u64) -> u64 {
        let mut z = self
            .0
            .wrapping_add(salt.wrapping_mul(0xD1B5_4A32_D192_ED03))
            .wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Pick one of `items` seeded by tile index and salt
    pub fn pick<'a, T>(&self, items: &'a [T], salt: u64) -> Option<&'a T> {
        true_!(!items.is_empty(), return None);
        items.get((self.seeded(salt) % items.len() as u64) as usize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldHeightIndex(pub u64);

impl From<WorldHeightIndex> for WorldTileIndex {
    fn from(value: WorldHeightIndex) -> Self {
        Self(value.0)
    }
}

impl WcfgFrom<WorldHeightIndex> for WorldTileIndex {
    fn from_(value: WorldHeightIndex, _: &WorldConfig) -> Self {
        Self(value.0)
    }
}

impl WcfgFrom<WorldTileIndex> for WorldTileIndex {
    fn from_(value: WorldTileIndex, _: &WorldConfig) -> Self {
        Self(value.0)
    }
}

impl From<WorldTileIndex> for WorldHeightIndex {
    fn from(value: WorldTileIndex) -> Self {
        Self(value.0)
    }
}

impl WcfgFrom<WorldTileIndex> for WorldHeightIndex {
    fn from_(value: WorldTileIndex, _: &WorldConfig) -> Self {
        Self(value.0)
    }
}

impl WcfgFrom<WorldTileIndex> for TileXy {
    fn from_(WorldTileIndex(i): WorldTileIndex, w: &WorldConfig) -> Self {
        let x = i % w.world_width();
        let y = i / w.world_width();
        Self(Xy(x, y))
    }
}

impl WcfgFrom<WorldTileIndex> for Xy {
    fn from_(WorldTileIndex(i): WorldTileIndex, w: &WorldConfig) -> Self {
        let x = i % w.world_width();
        let y = i / w.world_width();
        Xy(x, y)
    }
}

impl WcfgFrom<WorldHeightIndex> for Xy {
    fn from_(WorldHeightIndex(i): WorldHeightIndex, w: &WorldConfig) -> Self {
        let x = i % w.world_width();
        let y = i / w.world_width();
        Xy(x, y)
    }
}

impl WcfgFrom<TileXy> for WorldTileIndex {
    fn from_(TileXy(Xy(x, y)): TileXy, w: &WorldConfig) -> Self {
        Self(y * w.world_width() + x)
    }
}

impl WcfgFrom<WorldVec2> for WorldTileIndex {
    fn from_(value: WorldVec2, w: &WorldConfig) -> Self {
        WorldTileIndex::from_(
            TileXy(Xy(
                value.x as u64 / w.geo_pixels_per_tile(),
                value.y as u64 / w.geo_pixels_per_tile(),
            )),
            w,
        )
    }
}

impl WcfgFrom<[f32; 2]> for TileXy {
    fn from_(value: [f32; 2], w: &WorldConfig) -> Self {
        Self(Xy(
            value[0] as u64 / w.geo_pixels_per_tile(),
            value[1] as u64 / w.geo_pixels_per_tile(),
        ))
    }
}

impl WcfgFrom<[f32; 3]> for TileXy {
    fn from_(value: [f32; 3], w: &WorldConfig) -> Self {
        Self(Xy(
            value[0] as u64 / w.geo_pixels_per_tile(),
            value[1] as u64 / w.geo_pixels_per_tile(),
        ))
    }
}

impl WcfgFrom<RegionXy> for TileXy {
    fn from_(value: RegionXy, w: &WorldConfig) -> Self {
        let x = value.0.0 * w.region_width();
        let y = value.0.1 * w.region_height();
        Self(Xy(x, y))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(999, 0, 999)]
    #[case(1000, 0, 1000)]
    #[case(1000, 1, 2000)]
    #[case(1001, 1, 2001)]
    pub fn test_world_tile_index(#[case] x: u64, #[case] y: u64, #[case] i: u64) {
        // Given

        let w = WorldConfig::new(1000, 1000);

        // When
        let index = WorldTileIndex::from_(TileXy(Xy(x, y)), &w);

        // Then
        assert_eq!(index.0, i);
    }
}
