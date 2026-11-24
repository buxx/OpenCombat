pub mod image;

use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_root::{WcfgFrom, WorldConfig};
use oc_utils::d2::{Direction, Xy};

// WARNING: AI generated
/// Tiles covered by a rectangle considering orientation.
pub fn oriented_rectangle_covered_tiles(
    center: [f32; 2],
    width: f32,
    length: f32,
    direction: Direction,
    w: &WorldConfig,
) -> Vec<WorldTileIndex> {
    let tile = w.geo_pixels_per_tile() as f32;
    let forward = {
        let norm = (direction.x * direction.x + direction.y * direction.y).sqrt();
        if norm > f32::EPSILON {
            [direction.x / norm, direction.y / norm]
        } else {
            [Direction::NORTH.x, Direction::NORTH.y]
        }
    };
    let lateral = [-forward[1], forward[0]];
    let (half_length, half_width) = (length / 2., width / 2.);

    // Axis aligned bounding box of the rectangle
    let extent_x = (forward[0] * half_length).abs() + (lateral[0] * half_width).abs();
    let extent_y = (forward[1] * half_length).abs() + (lateral[1] * half_width).abs();
    let range = |pos: f32, extent: f32, count: u64| -> Option<(u64, u64)> {
        let min = ((pos - extent) / tile).floor().max(0.);
        let max = (((pos + extent) / tile).ceil() - 1.).min(count as f32 - 1.);
        (min <= max).then_some((min as u64, max as u64))
    };
    let Some((min_x, max_x)) = range(center[0], extent_x, w.world_width()) else {
        return vec![];
    };
    let Some((min_y, max_y)) = range(center[1], extent_y, w.world_height()) else {
        return vec![];
    };

    // Separating axis test on rectangle axes (tiles axes are covered by bounding box)
    let overlap = |tile_center: [f32; 2], axis: [f32; 2], extent: f32| {
        let distance =
            ((tile_center[0] - center[0]) * axis[0] + (tile_center[1] - center[1]) * axis[1]).abs();
        let tile_extent = tile / 2. * (axis[0].abs() + axis[1].abs());
        distance < extent + tile_extent
    };

    let mut result = vec![];
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let tile_center = [(x as f32 + 0.5) * tile, (y as f32 + 0.5) * tile];
            if overlap(tile_center, forward, half_length)
                && overlap(tile_center, lateral, half_width)
            {
                result.push(WorldTileIndex::from_(TileXy(Xy(x, y)), w));
            }
        }
    }

    result
}
