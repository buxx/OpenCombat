use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use ggez::GameError;
use ggez::GameResult;
use tiled::{
    parse_with_path, Image as TiledImage, Image, ImageLayer, Layer, LayerData, Map as TiledMap,
    ObjectGroup, Orientation, PropertyValue, Tileset,
};

use crate::map::decor::{Decor, DecorTile};
use crate::map::terrain::{Terrain, TerrainTile};
use crate::map::util::{
    extract_gids, extract_image_from_image_layer, extract_image_from_tileset, extract_layer,
    extract_objects, extract_tileset, extract_tileset_images, extract_tilesets_containing_gids,
    get_tileset_i_for_gid,
};

pub mod decor;
pub mod terrain;
pub mod util;

pub struct Map {
    pub tiled_map: TiledMap,
    pub background_image: TiledImage,
    pub interiors_objects: ObjectGroup,
    pub interiors_image: TiledImage,
    pub terrain: Terrain,
    pub decor: Decor,
}

impl Map {
    pub fn new(map_file_path: &Path) -> GameResult<Self> {
        let map_file = File::open(map_file_path)?;
        let map_file_reader = BufReader::new(map_file);
        let tiled_map = match parse_with_path(map_file_reader, map_file_path) {
            Ok(map) => map,
            Err(e) => {
                return GameResult::Err(GameError::ResourceLoadError(format!(
                    "Fail to parse map: {:?}",
                    e
                )))
            }
        };

        if &tiled_map.orientation != &Orientation::Orthogonal {
            return GameResult::Err(GameError::ResourceLoadError(
                "Map must be orthogonal orientation".to_string(),
            ));
        }

        // Background and interiors
        let background_image = extract_image_from_image_layer(&tiled_map, "background")?;
        let interiors_objects = extract_objects(&tiled_map, "interiors")?;
        let interiors_image = extract_image_from_image_layer(&tiled_map, "interiors")?;

        // Terrain
        let terrain_tileset: Tileset = extract_tileset(&tiled_map, "terrain")?;
        let terrain_layer: Layer = extract_layer(&tiled_map, "terrain")?;
        let terrain_image = extract_image_from_tileset(&terrain_tileset)?;

        let mut terrain_tiles: HashMap<(u32, u32), TerrainTile> = HashMap::new();
        match &terrain_layer.tiles {
            LayerData::Finite(layer_tiles) => {
                for (x, tiles_row) in layer_tiles.iter().enumerate() {
                    for (y, layer_tile) in tiles_row.iter().enumerate() {
                        // FIXME BS NOW: et si gid = 0 ?
                        let tile = terrain::get_tile_from_terrain_tileset_with_id(
                            &terrain_tileset,
                            layer_tile.gid,
                            terrain_image.width as u32,
                            terrain_image.height as u32,
                        )?;
                        terrain_tiles.insert((y as u32, x as u32), tile);
                    }
                }
            }
            LayerData::Infinite(_) => {
                return GameResult::Err(GameError::ResourceLoadError(
                    "Terrain layer must be finite".to_string(),
                ))
            }
        }

        let terrain = Terrain::new(terrain_tileset, terrain_layer, terrain_image, terrain_tiles);

        // Decor
        let mut decor_tiles: HashMap<(u32, u32), DecorTile> = HashMap::new();
        let decor_layer: Layer = extract_layer(&tiled_map, "decor")?;
        let decor_tile_gids = extract_gids(&decor_layer)?;
        let decor_tilesets: Vec<Tileset> =
            extract_tilesets_containing_gids(&tiled_map, decor_tile_gids);
        let decor_images: Vec<TiledImage> = extract_tileset_images(&decor_tilesets)?;

        match &decor_layer.tiles {
            LayerData::Finite(layer_tiles) => {
                for (x, tiles_row) in layer_tiles.iter().enumerate() {
                    for (y, layer_tile) in tiles_row.iter().enumerate() {
                        if layer_tile.gid == 0 {
                            continue;
                        }

                        let tileset_i = get_tileset_i_for_gid(layer_tile.gid, &decor_tilesets)
                            .expect("gid must be find");
                        let tileset = decor_tilesets
                            .get(tileset_i)
                            .expect("Decor tileset must be here");
                        let image = decor_images.get(tileset_i).expect("Image must exist");

                        let tiled_tile_id = layer_tile.gid - tileset.first_gid;
                        let tile_width = tileset.tile_width;
                        let tile_height = tileset.tile_height;
                        let relative_tile_width = tile_width as f32 / image.width as f32;
                        let relative_tile_height = tile_height as f32 / image.height as f32;
                        let len_by_width = image.width as u32 / tile_width;
                        let tile_y = tiled_tile_id / len_by_width;
                        let tile_x = tiled_tile_id - (tile_y * len_by_width);

                        let tile = DecorTile::new(
                            tileset_i,
                            tile_width,
                            tile_height,
                            relative_tile_width,
                            relative_tile_height,
                            tile_x,
                            tile_y,
                        );
                        decor_tiles.insert((y as u32, x as u32), tile);
                    }
                }
            }
            LayerData::Infinite(_) => {
                return GameResult::Err(GameError::ResourceLoadError(
                    "Decor layer must be finite".to_string(),
                ))
            }
        }

        let decor = Decor::new(decor_layer, decor_tilesets, decor_images, decor_tiles);

        GameResult::Ok(Map {
            tiled_map: tiled_map.clone(),
            background_image: background_image.clone(),
            interiors_objects,
            interiors_image,
            terrain,
            decor,
        })
    }
}
