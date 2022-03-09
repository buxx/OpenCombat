use core::option::Option::{None, Some};
use ggez::error::{GameError, GameResult};
use tiled::{Image as TiledImage, Layer, LayerData, Map as TiledMap, ObjectGroup, Tileset};

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
