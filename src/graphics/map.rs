use std::path::Path;

use ggez::{
    graphics::{spritebatch::SpriteBatch, DrawParam, Image, Rect},
    Context, GameResult,
};

use crate::{
    map::{util::extract_image_from_tileset, Map},
    types::*,
};

pub fn get_map_background_batch(ctx: &mut Context, map: &Map) -> GameResult<SpriteBatch> {
    let map_background_image = Image::new(
        ctx,
        &Path::new(&format!(
            "/maps/{}/{}",
            &map.id, &map.background_image.source
        )),
    )?;
    let mut map_background_batch = SpriteBatch::new(map_background_image);
    // This batch will never change, add draw param once
    map_background_batch.add(
        DrawParam::new()
            .src(Rect::new(0.0, 0.0, 1.0, 1.0))
            .dest(ScenePoint::new(0.0, 0.0).to_vec2()),
    );

    Ok(map_background_batch)
}

pub fn get_map_interiors_batch(ctx: &mut Context, map: &Map) -> GameResult<SpriteBatch> {
    let map_interiors_image = Image::new(
        ctx,
        &Path::new(&format!(
            "/maps/{}/{}",
            &map.id, &map.interiors_image.source
        )),
    )?;
    let mut map_interiors_batch = SpriteBatch::new(map_interiors_image);
    // This batch will never change, add draw param once
    map_interiors_batch.add(
        DrawParam::new()
            .src(Rect::new(0.0, 0.0, 1.0, 1.0))
            .dest(ScenePoint::new(0.0, 0.0).to_vec2()),
    );

    Ok(map_interiors_batch)
}

pub fn get_map_decor_batch(ctx: &mut Context, map: &Map) -> GameResult<Vec<SpriteBatch>> {
    let mut map_decor_batches = vec![];
    for decor_tileset in map.decor.tilesets.iter() {
        let decor_tiled_image = extract_image_from_tileset(decor_tileset)?;
        let decor_image = Image::new(
            ctx,
            format!("/maps/{}/{}", map.id, decor_tiled_image.source),
        )?;
        let batch = SpriteBatch::new(decor_image);
        map_decor_batches.push(batch);
    }

    for decor_batch in map_decor_batches.iter_mut() {
        // TODO : Why this clear ?
        decor_batch.clear();
    }

    for ((grid_x, grid_y), tile) in map.decor.tiles.iter() {
        let decor_batch = map_decor_batches
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
            DrawParam::new()
                .src(Rect::new(
                    src_x,
                    src_y,
                    tile.relative_tile_width,
                    tile.relative_tile_height,
                ))
                .dest(ScenePoint::new(dest_x, dest_y).to_vec2()),
        );
    }

    Ok(map_decor_batches)
}
