use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::Image;
use ggez::graphics::MeshBuilder;
use ggez::Context;
use ggez::GameError;
use ggez::GameResult;
use strum::IntoEnumIterator;
use tiled::{
    parse_with_path, Image as TiledImage, Layer, LayerData, Map as TiledMap, ObjectGroup,
    Orientation, Tileset,
};

use crate::map::decor::{Decor, DecorTile};
use crate::map::terrain::{Terrain, TerrainTile};
use crate::map::util::{
    extract_gids, extract_image_from_image_layer, extract_image_from_tileset, extract_layer,
    extract_objects, extract_tileset, extract_tileset_images, extract_tilesets_containing_gids,
    get_tileset_i_for_gid,
};
use crate::physics::path::Direction;
use crate::physics::path::PathMode;
use crate::types::*;
use crate::RESOURCE_PATH;
use core::cmp;

use self::util::create_debug_terrain_opacity_mesh_builder;
use self::util::update_terrain_batch;

pub mod decor;
pub mod terrain;
pub mod util;

pub struct Map {
    pub id: String,
    pub tiled_map: TiledMap,
    pub background_image: TiledImage,
    pub interiors_objects: ObjectGroup,
    pub interiors_image: TiledImage,
    pub terrain: Terrain,
    pub decor: Decor,
    pub terrain_grid_width: u32,
    pub terrain_grid_height: u32,
    pub debug_terrain_batch: Option<SpriteBatch>,
    pub debug_terrain_opacity_mesh_builder: Option<MeshBuilder>,
}

impl Map {
    pub fn new(ctx: &mut Context, id: &str) -> GameResult<Self> {
        let map_file_path_string = format!("{}/maps/{}/{}.tmx", RESOURCE_PATH, id, id);
        let map_file_path = &Path::new(&map_file_path_string);
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
        let mut terrain_grid_width: u32 = 0;
        let mut terrain_grid_height: u32 = 0;
        match &terrain_layer.tiles {
            LayerData::Finite(layer_tiles) => {
                for (x, tiles_row) in layer_tiles.iter().enumerate() {
                    terrain_grid_height = cmp::max(terrain_grid_height, x as u32);
                    for (y, layer_tile) in tiles_row.iter().enumerate() {
                        terrain_grid_width = cmp::max(terrain_grid_width, y as u32);
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

        let terrain = Terrain::new(
            terrain_tileset,
            terrain_layer,
            terrain_image.clone(),
            terrain_tiles,
        );
        let decor = Decor::new(decor_layer, decor_tilesets, decor_images, decor_tiles);

        let mut map = Map {
            id: id.to_string(),
            tiled_map: tiled_map.clone(),
            background_image: background_image.clone(),
            interiors_objects,
            interiors_image,
            terrain: terrain.clone(),
            decor,
            terrain_grid_width,
            terrain_grid_height,
            debug_terrain_batch: None,
            debug_terrain_opacity_mesh_builder: None,
        };

        let mut debug_terrain_batch = SpriteBatch::new(Image::new(
            ctx,
            format!("/maps/map1/{}", terrain_image.source),
        )?);
        debug_terrain_batch = update_terrain_batch(debug_terrain_batch, &map);
        map.debug_terrain_batch = Some(debug_terrain_batch);
        let debug_terrain_opacity_mesh_builder = create_debug_terrain_opacity_mesh_builder(&map)?;
        map.debug_terrain_opacity_mesh_builder = Some(debug_terrain_opacity_mesh_builder);

        GameResult::Ok(map)
    }

    pub fn successors(
        &self,
        from: &(GridPoint, Direction),
        path_mode: &PathMode,
    ) -> Vec<((GridPoint, Direction), i32)> {
        let mut successors = vec![];

        for direction in Direction::iter() {
            let (mod_x, mod_y) = direction.modifier();
            let new_x = from.0.x + mod_x;
            let new_y = from.0.y + mod_y;

            // Don't care ifd outside map
            if new_x < 0 || new_y < 0 {
                continue;
            }

            // If in map
            if let Some(next_tile) = self.terrain.tiles.get(&(new_x as u32, new_y as u32)) {
                if path_mode.include_vehicles() {
                    if next_tile.block_vehicle {
                        continue;
                    }

                    match path_mode {
                        PathMode::Drive(size) => {
                            if !self.point_allow_vehicle(&GridPoint::new(new_x, new_y), size) {
                                continue;
                            }
                        }
                        _ => {}
                    }
                }

                let cost = match path_mode {
                    PathMode::Walk => next_tile.pedestrian_cost,
                    PathMode::Drive(_size) => from.1.angle_cost(&direction),
                };

                successors.push(((GridPoint::new(new_x, new_y), direction), cost))
            }
        }

        successors
    }

    pub fn contains(&self, grid_point: &GridPoint) -> bool {
        grid_point.x >= 0
            && grid_point.y >= 0
            && grid_point.x < self.terrain_grid_width as i32
            && grid_point.y < self.terrain_grid_height as i32
    }

    pub fn grid_point_from_world_point(&self, world_point: &WorldPoint) -> GridPoint {
        let x = world_point.x as u32 / self.terrain.tileset.tile_width;
        let y = world_point.y as u32 / self.terrain.tileset.tile_height;
        GridPoint::new(x as i32, y as i32)
    }

    fn point_allow_vehicle(&self, point: &GridPoint, size: &VehicleSize) -> bool {
        let half = (size.0 / 2) as i32;
        let start_x = point.x - half;
        let end_x = point.x + half;
        let start_y = point.y - half;
        let end_y = point.y + half;

        for x in start_x..end_x {
            for y in start_y..end_y {
                if let Some(tile) = self.terrain.tiles.get(&(x as u32, y as u32)) {
                    if tile.block_vehicle {
                        return false;
                    }
                }
            }
        }

        return true;
    }
}
