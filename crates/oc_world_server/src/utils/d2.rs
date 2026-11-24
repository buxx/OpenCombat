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
    use oc_utils::d2::Direction;
    use oc_world::utils::oriented_rectangle_covered_tiles;

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
    fn test_oriented_rectangle_cover_tiles_aligned() {
        // Given: 1 pixel tiles, rectangle 1 wide and 3 long facing north, centered on tile (2, 2)
        let w = WorldConfig::new(5, 5).with_geo_pixels_per_tile(1);

        // When
        let tiles = oriented_rectangle_covered_tiles([2.5, 2.5], 1., 3., Direction::NORTH, &w);

        // Then
        assert_eq!(
            tiles,
            vec![WorldTileIndex(7), WorldTileIndex(12), WorldTileIndex(17)]
        );
    }

    #[test]
    fn test_oriented_rectangle_cover_tiles_rotated() {
        // Given: same rectangle facing east
        let w = WorldConfig::new(5, 5).with_geo_pixels_per_tile(1);

        // When
        let tiles = oriented_rectangle_covered_tiles([2.5, 2.5], 1., 3., Direction::EST, &w);

        // Then
        assert_eq!(
            tiles,
            vec![WorldTileIndex(11), WorldTileIndex(12), WorldTileIndex(13)]
        );
    }

    #[test]
    fn test_oriented_rectangle_cover_tiles_diagonal() {
        // Given: thin rectangle along the diagonal, don't cover opposite corners
        let w = WorldConfig::new(5, 5).with_geo_pixels_per_tile(1);
        let direction = Direction::new(1., 1.);

        // When
        let tiles = oriented_rectangle_covered_tiles([2.5, 2.5], 0.2, 4., direction, &w);

        // Then
        assert!(tiles.contains(&WorldTileIndex(6)));
        assert!(tiles.contains(&WorldTileIndex(12)));
        assert!(tiles.contains(&WorldTileIndex(18)));
        assert!(!tiles.contains(&WorldTileIndex(8)));
        assert!(!tiles.contains(&WorldTileIndex(16)));
    }

    #[test]
    fn test_oriented_rectangle_cover_tiles_out_of_world() {
        // Given
        let w = WorldConfig::new(5, 5).with_geo_pixels_per_tile(1);

        // When
        let tiles = oriented_rectangle_covered_tiles([0.5, 0.5], 1., 3., Direction::NORTH, &w);

        // Then
        assert_eq!(tiles, vec![WorldTileIndex(0), WorldTileIndex(5)]);
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
