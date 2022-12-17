use core::option::Option::{None, Some};
use ggez::{
    error::{GameError, GameResult},
    graphics::{spritebatch::SpriteBatch, Color, DrawMode, DrawParam, MeshBuilder, Rect},
};
use tiled::{Image as TiledImage, Layer, LayerData, Map as TiledMap, ObjectGroup, Tileset};

use crate::{config::COVER_DISTANCE, types::*, utils::grid_points_for_square};

use super::{terrain::TerrainTile, Map};

pub fn extract_image_from_image_layer(
    tiled_map: &TiledMap,
    layer_name: &str,
) -> GameResult<TiledImage> {
    for image_layer in tiled_map.image_layers.iter() {
        if image_layer.name == layer_name {
            return match &image_layer.image {
                None => GameResult::Err(GameError::ResourceLoadError(format!(
                    "No image in image layer {}",
                    layer_name
                ))),
                Some(image) => GameResult::Ok(image.clone()),
            };
        }
    }

    return GameResult::Err(GameError::ResourceLoadError(format!(
        "Image layer {} not found in map",
        layer_name
    )));
}

pub fn extract_image_from_tileset(tileset: &Tileset) -> GameResult<TiledImage> {
    match tileset.images.first() {
        None => {
            return GameResult::Err(GameError::ResourceLoadError(
                "No image found in given tileset".to_string(),
            ))
        }
        Some(terrain_image) => GameResult::Ok(terrain_image.clone()),
    }
}

pub fn extract_tileset(tiled_map: &TiledMap, tileset_name: &str) -> GameResult<Tileset> {
    match tiled_map
        .tilesets
        .clone()
        .into_iter()
        .filter(|t| t.name == tileset_name)
        .collect::<Vec<Tileset>>()
        .first()
    {
        None => {
            return GameResult::Err(GameError::ResourceLoadError(format!(
                "No tileset {} found in map",
                tileset_name
            )))
        }
        Some(tileset) => GameResult::Ok(tileset.clone()),
    }
}

pub fn extract_objects(tiled_map: &TiledMap, objects_name: &str) -> GameResult<ObjectGroup> {
    match tiled_map
        .object_groups
        .clone()
        .into_iter()
        .filter(|g| g.name == objects_name)
        .collect::<Vec<ObjectGroup>>()
        .first()
    {
        None => {
            return GameResult::Err(GameError::ResourceLoadError(format!(
                "No objects {} found in map",
                objects_name
            )))
        }
        Some(object_groups) => GameResult::Ok(object_groups.clone()),
    }
}

pub fn extract_layer(tiled_map: &TiledMap, layer_name: &str) -> GameResult<Layer> {
    match tiled_map
        .layers
        .clone()
        .into_iter()
        .filter(|l| l.name == layer_name)
        .collect::<Vec<Layer>>()
        .first()
    {
        None => {
            return GameResult::Err(GameError::ResourceLoadError(format!(
                "No {} layer found in map",
                layer_name
            )))
        }
        Some(layer) => GameResult::Ok(layer.clone()),
    }
}

pub fn extract_gids(layer: &Layer) -> GameResult<Vec<u32>> {
    let mut gids: Vec<u32> = vec![];

    match &layer.tiles {
        LayerData::Finite(layer_tiles) => {
            for (_, tiles_row) in layer_tiles.iter().enumerate() {
                for (_, layer_tile) in tiles_row.iter().enumerate() {
                    if !gids.contains(&layer_tile.gid) {
                        gids.push(layer_tile.gid);
                    }
                }
            }
        }
        LayerData::Infinite(_) => {
            return GameResult::Err(GameError::ResourceLoadError(
                "Layer must be finite".to_string(),
            ))
        }
    }

    GameResult::Ok(gids)
}

pub fn get_tileset_i_for_gid(gid: u32, tilesets: &Vec<Tileset>) -> GameResult<usize> {
    for (i, tileset) in tilesets.iter().enumerate() {
        if gid >= tileset.first_gid && gid < tileset.first_gid + tileset.tilecount.unwrap() {
            return GameResult::Ok(i);
        }
    }

    return GameResult::Err(GameError::ResourceLoadError(format!(
        "gid {} not found for given tilesets",
        gid
    )));
}

pub fn extract_tilesets_containing_gids(
    tiled_map: &TiledMap,
    decor_tile_gids: Vec<u32>,
) -> Vec<Tileset> {
    let mut tilesets: Vec<Tileset> = vec![];

    for tileset in tiled_map.tilesets.iter() {
        for decor_tile_gid in &decor_tile_gids {
            if *decor_tile_gid >= tileset.first_gid
                && *decor_tile_gid < tileset.first_gid + tileset.tilecount.unwrap()
            {
                if !tilesets.contains(&tileset) {
                    tilesets.push(tileset.clone());
                }
            }
        }
    }

    tilesets
}

pub fn extract_tileset_images(tilesets: &Vec<Tileset>) -> GameResult<Vec<TiledImage>> {
    let mut images: Vec<TiledImage> = vec![];

    for tileset in tilesets.iter() {
        images.push(extract_image_from_tileset(tileset)?)
    }

    GameResult::Ok(images)
}

pub fn find_cover_grid_point(
    from_grid_point: &GridPoint,
    map: &Map,
    exclude_grid_points: &Vec<GridPoint>,
) -> Option<(GridPoint, Vec<GridPoint>)> {
    let mut tiles: Vec<(GridPoint, &TerrainTile)> = vec![];
    if let Some(tile) = map
        .terrain
        .tiles
        .get(&(from_grid_point.x as u32, from_grid_point.y as u32))
    {
        tiles.push((from_grid_point.clone(), tile))
    }
    let grid_points_for_square =
        grid_points_for_square(&from_grid_point, COVER_DISTANCE, COVER_DISTANCE);
    for grid_point in grid_points_for_square {
        if let Some(tile) = map
            .terrain
            .tiles
            .get(&(grid_point.x as u32, grid_point.y as u32))
        {
            tiles.push((grid_point, tile))
        }
    }
    tiles.sort_by(|(_, tile_a), (_, tile_b)| tile_a.opacity.partial_cmp(&tile_b.opacity).unwrap());

    for (grid_point, _) in tiles.iter().rev() {
        if !exclude_grid_points.contains(grid_point) {
            let grid_points = tiles
                .iter()
                .map(|(p, _)| p.clone())
                .collect::<Vec<GridPoint>>();
            return Some((grid_point.clone(), grid_points));
        }
    }

    None
}

pub fn update_terrain_batch(mut terrain_batch: SpriteBatch, map: &Map) -> SpriteBatch {
    terrain_batch.clear();
    for ((grid_x, grid_y), tile) in map.terrain.tiles.iter() {
        let src_x = tile.tile_x as f32 * tile.relative_tile_width;
        let src_y = tile.tile_y as f32 * tile.relative_tile_height;
        let dest_x = *grid_x as f32 * tile.tile_width as f32;
        let dest_y = *grid_y as f32 * tile.tile_height as f32;
        terrain_batch.add(
            DrawParam::new()
                .src(Rect::new(
                    src_x,
                    src_y,
                    tile.relative_tile_width,
                    tile.relative_tile_height,
                ))
                .dest(WorldPoint::new(dest_x, dest_y).to_vec2()),
        );
    }

    terrain_batch
}

pub fn create_debug_terrain_opacity_mesh_builder(map: &Map) -> GameResult<MeshBuilder> {
    let mut debug_terrain_opacity_mesh = MeshBuilder::new();
    for ((grid_x, grid_y), tile) in map.terrain.tiles.iter() {
        let dest_x = *grid_x as f32 * tile.tile_width as f32;
        let dest_y = *grid_y as f32 * tile.tile_height as f32;
        let color_modifier = 0.6 * tile.opacity * 3.0;
        debug_terrain_opacity_mesh.rectangle(
            DrawMode::fill(),
            Rect::new(
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
