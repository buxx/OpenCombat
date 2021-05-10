use ggez::graphics::{Color, DrawMode, MeshBuilder};
use ggez::{graphics, GameResult};

use crate::map::Map;
use crate::ScenePoint;

pub fn update_terrain_batch(
    mut terrain_batch: graphics::spritebatch::SpriteBatch,
    map: &Map,
) -> graphics::spritebatch::SpriteBatch {
    terrain_batch.clear();
    for ((grid_x, grid_y), tile) in map.tiles.iter() {
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
    for ((grid_x, grid_y), tile) in map.tiles.iter() {
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
