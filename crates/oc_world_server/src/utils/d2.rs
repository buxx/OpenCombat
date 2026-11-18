use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_root::{WcfgFrom, WorldConfig, geo::WorldVec2, physics::Meters};
use oc_utils::d2::Xy;

pub fn shape_from_tile(i: WorldTileIndex, radius: u64, w: &WorldConfig) -> Vec<WorldTileIndex> {
    let xy = TileXy::from_(i, w);
    let min_x = xy.0.0.saturating_sub(radius);
    let min_y = xy.0.1.saturating_sub(radius);
    let max_x = xy.0.0 + radius;
    let max_y = xy.0.1 + radius;
    let clamped_max_x = max_x.min(w.world_width().saturating_sub(1));
    let clamped_max_y = max_y.min(w.world_height().saturating_sub(1));

    let mut result =
        Vec::with_capacity(((clamped_max_x - min_x + 1) * (clamped_max_y - min_y + 1)) as usize);

    for y in min_y..=clamped_max_y {
        for x in min_x..=clamped_max_x {
            let i = WorldTileIndex::from_(TileXy(Xy(x, y)), w);
            result.push(i);
        }
    }

    result
}

pub fn random_position_around(w: &WorldConfig, center: WorldVec2, radius: Meters) -> WorldVec2 {
    let radius = radius.0 * w.geo_pixels_per_meters();
    let distance = radius * fastrand::f32().sqrt();
    let angle = fastrand::f32() * std::f32::consts::TAU;
    let x = center.x + distance * angle.cos();
    let y = center.y + distance * angle.sin();

    WorldVec2::new(
        x.clamp(0., (w.world_width_pixels() - 1) as f32),
        y.clamp(0., (w.world_height_pixels() - 1) as f32),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_from_tile_top_left_corner() {
        // Given
        let w = WorldConfig::new(5, 5);
        let i = WorldTileIndex(0);

        // When
        let tiles = shape_from_tile(i, 2, &w);

        // Then
        assert_eq!(
            tiles,
            vec![
                WorldTileIndex(0),
                WorldTileIndex(1),
                WorldTileIndex(2),
                WorldTileIndex(5),
                WorldTileIndex(6),
                WorldTileIndex(7),
                WorldTileIndex(10),
                WorldTileIndex(11),
                WorldTileIndex(12)
            ]
        );
    }

    #[test]
    fn test_shape_from_tile_bottom_right_corner() {
        // Given
        let w = WorldConfig::new(5, 5);
        let i = WorldTileIndex(24);

        // When
        let tiles = shape_from_tile(i, 2, &w);

        // Then
        assert_eq!(
            tiles,
            vec![
                WorldTileIndex(12),
                WorldTileIndex(13),
                WorldTileIndex(14),
                WorldTileIndex(17),
                WorldTileIndex(18),
                WorldTileIndex(19),
                WorldTileIndex(22),
                WorldTileIndex(23),
                WorldTileIndex(24)
            ]
        );
    }

    #[test]
    fn test_shape_from_tile_center() {
        // Given
        let w = WorldConfig::new(5, 5);
        let i = WorldTileIndex(12);

        // When
        let tiles = shape_from_tile(i, 1, &w);

        // Then
        assert_eq!(
            tiles,
            vec![
                WorldTileIndex(6),
                WorldTileIndex(7),
                WorldTileIndex(8),
                WorldTileIndex(11),
                WorldTileIndex(12),
                WorldTileIndex(13),
                WorldTileIndex(16),
                WorldTileIndex(17),
                WorldTileIndex(18)
            ]
        );
    }
}
