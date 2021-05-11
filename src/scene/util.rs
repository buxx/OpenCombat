use ggez::graphics::{Color, DrawMode, MeshBuilder};
use ggez::{graphics, GameResult};

use crate::map::Map;
use crate::ScenePoint;
use tiled::Tileset;

pub fn update_terrain_batch(
    mut terrain_batch: graphics::spritebatch::SpriteBatch,
    map: &Map,
) -> graphics::spritebatch::SpriteBatch {
    terrain_batch.clear();
    for ((grid_x, grid_y), tile) in map.terrain.tiles.iter() {
        // FIXME pre compute these data ?
        let src_x = tile.tile_x as f32 * tile.relative_tile_width;
        let src_y = tile.tile_y as f32 * tile.relative_tile_height;
        let dest_x = *grid_x as f32 * tile.tile_width as f32;
        let dest_y = *grid_y as f32 * tile.tile_height as f32;
        terrain_batch.add(
            graphics::DrawParam::new()
                .src(graphics::Rect::new(
                    src_x,
                    src_y,
                    tile.relative_tile_width,
                    tile.relative_tile_height,
                ))
                .dest(ScenePoint::new(dest_x, dest_y)),
        );
    }

    terrain_batch
}

pub fn create_debug_terrain_opacity_mesh_builder(map: &Map) -> GameResult<MeshBuilder> {
    let mut debug_terrain_opacity_mesh = MeshBuilder::new();
    for ((grid_x, grid_y), tile) in map.terrain.tiles.iter() {
        let dest_x = *grid_x as f32 * tile.tile_width as f32;
        let dest_y = *grid_y as f32 * tile.tile_height as f32;
        let color_modifier = 0.6 * tile.opacity;
        debug_terrain_opacity_mesh.rectangle(
            DrawMode::fill(),
            graphics::Rect::new(
                dest_x,
                dest_y,
                tile.tile_width as f32,
                tile.tile_height as f32,
            ),
            Color {
                r: 0.4 - color_modifier,
                g: 0.4 - color_modifier,
                b: 0.4 - color_modifier,
                a: 1.0,
            },
        )?;
    }
    Ok(debug_terrain_opacity_mesh)
}

pub fn update_decor_batches(
    decor_batches: &mut Vec<graphics::spritebatch::SpriteBatch>,
    map: &Map,
) {
    for decor_batch in decor_batches.iter_mut() {
        decor_batch.clear();
    }

    for ((grid_x, grid_y), tile) in map.decor.tiles.iter() {
        let decor_batch = decor_batches
            .get_mut(tile.tileset_i)
            .expect("Batch must be here");

        // Tiled draw from bottom left but we draw from top left, so compute a decal
        let dest_decal = tile.tile_height as f32 - map.terrain.tileset.tile_height as f32;
        let src_x = tile.tile_x as f32 * tile.relative_tile_width;
        let src_y = tile.tile_y as f32 * tile.relative_tile_height;
        // Destination computation refer to terrain grid (map.terrain.tileset)
        let dest_x = *grid_x as f32 * map.terrain.tileset.tile_width as f32;
        let dest_y = (*grid_y as f32 * map.terrain.tileset.tile_height as f32) - dest_decal;

        decor_batch.add(
            graphics::DrawParam::new()
                .src(graphics::Rect::new(
                    src_x,
                    src_y,
                    tile.relative_tile_width,
                    tile.relative_tile_height,
                ))
                .dest(ScenePoint::new(dest_x, dest_y)),
        );
    }
}
