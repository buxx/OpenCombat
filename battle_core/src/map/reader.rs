use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use oc_core::spawn::{ParseOriginDirectionError, SpawnZoneName};
use tiled::{
    FiniteTileLayer, Image, ImageLayer, Layer, LayerType, Loader, Map as TiledMap, ObjectLayer,
    TileLayer, Tileset,
};

use crate::game::flag::{Flag, FlagName};

use super::{
    decor::{Decor, DecorTile},
    interior::Interior,
    spawn::SpawnZone,
    terrain::{TerrainTile, TerrainTileError},
    Map,
};

type DecorTilesets = (Vec<Arc<Tileset>>, HashMap<usize, usize>);

const BACKGROUND_IMAGE_LAYER_NAME: &str = "background_image";
const INTERIORS_IMAGE_LAYER_NAME: &str = "interiors_image";
const INTERIORS_ZONES_LAYER_NAME: &str = "interiors_zones";
const SPAWN_ZONES_LAYER_NAME: &str = "spawn_zones";
const FLAGS_LAYER_NAME: &str = "flags";
const DECOR_LAYER_NAME: &str = "decor";
const TERRAIN_LAYER_NAME: &str = "terrain";
const TERRAIN_TILESET_NAME: &str = "terrain";
const TILE_ID_PROPERTY_KEY: &str = "ID";

#[derive(Debug)]
pub enum MapReaderError {
    MapNotFound(String),
    LayerNotFound(String),
    InvalidLayer(String),
    TileSetNotFound(String),
    InvalidTileSet(String),
    TileError(String),
    TerrainTileError(TerrainTileError),
}

impl From<TerrainTileError> for MapReaderError {
    fn from(error: TerrainTileError) -> Self {
        Self::TerrainTileError(error)
    }
}

impl From<ParseOriginDirectionError> for MapReaderError {
    fn from(value: ParseOriginDirectionError) -> Self {
        Self::InvalidLayer(format!("Invalid origin direction : '{}'", value))
    }
}

impl Display for MapReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapReaderError::MapNotFound(error) => {
                f.write_str(&format!("Map not found : {}", error))
            }
            MapReaderError::LayerNotFound(error) => {
                f.write_str(&format!("Map not found : {}", error))
            }
            MapReaderError::InvalidLayer(error) => {
                f.write_str(&format!("Invalid layer : {}", error))
            }
            MapReaderError::TileSetNotFound(error) => {
                f.write_str(&format!("Tileset not found : {}", error))
            }
            MapReaderError::InvalidTileSet(error) => {
                f.write_str(&format!("Invalid tileset : {}", error))
            }
            MapReaderError::TileError(error) => f.write_str(&format!("Tile error : {}", error)),
            MapReaderError::TerrainTileError(error) => {
                f.write_str(&format!("Terrain tile error : {}", error))
            }
        }
    }
}

pub struct MapReader {
    resources: PathBuf,
    name: String,
    map: TiledMap,
}

impl MapReader {
    pub fn new(name: &str, resources: &Path) -> Result<Self, MapReaderError> {
        let map_file_path = format!("{}/maps/{}/{}.tmx", &resources.display(), name, name);
        let mut loader = Loader::new();

        let map = match loader.load_tmx_map(&map_file_path) {
            Ok(map) => map,
            Err(error) => {
                return Result::Err(MapReaderError::MapNotFound(format!(
                    "Failed to load map {} : {}",
                    map_file_path, error
                )))
            }
        };

        Ok(Self {
            resources: resources.to_path_buf(),
            name: name.to_string(),
            map,
        })
    }

    fn layer(&self, name: &str) -> Result<Layer, MapReaderError> {
        match self
            .map
            .layers()
            .filter(|layer| layer.name == name)
            .collect::<Vec<Layer>>()
            .first()
        {
            Some(layer) => Ok(*layer),
            None => Result::Err(MapReaderError::LayerNotFound(format!(
                "Failed to find layer '{}' in map {}",
                name, self.name,
            ))),
        }
    }

    fn background_image_layer(&self) -> Result<ImageLayer, MapReaderError> {
        match self.layer(BACKGROUND_IMAGE_LAYER_NAME)?.layer_type() {
            LayerType::ImageLayer(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map {} is not an image layer",
                BACKGROUND_IMAGE_LAYER_NAME, self.name,
            ))),
        }
    }

    fn background_image(&self) -> Result<Image, MapReaderError> {
        match &self.background_image_layer()?.image {
            Some(image) => Ok(image.clone()),
            None => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map {} must contains image",
                BACKGROUND_IMAGE_LAYER_NAME, self.name,
            ))),
        }
    }

    fn interiors_image_layer(&self) -> Result<ImageLayer, MapReaderError> {
        match self.layer(INTERIORS_IMAGE_LAYER_NAME)?.layer_type() {
            LayerType::ImageLayer(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map {} is not an image layer",
                INTERIORS_IMAGE_LAYER_NAME, self.name,
            ))),
        }
    }

    fn interiors_image(&self) -> Result<Image, MapReaderError> {
        match &self.interiors_image_layer()?.image {
            Some(image) => Ok(image.clone()),
            None => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map {} must contains image",
                INTERIORS_IMAGE_LAYER_NAME, self.name,
            ))),
        }
    }

    fn interiors_zones_layer(&self) -> Result<ObjectLayer, MapReaderError> {
        match self.layer(INTERIORS_ZONES_LAYER_NAME)?.layer_type() {
            LayerType::ObjectLayer(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map {} is not an object layer",
                INTERIORS_ZONES_LAYER_NAME, self.name,
            ))),
        }
    }

    fn spawn_zones_layer(&self) -> Result<ObjectLayer, MapReaderError> {
        match self.layer(SPAWN_ZONES_LAYER_NAME)?.layer_type() {
            LayerType::ObjectLayer(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map {} is not an object layer",
                SPAWN_ZONES_LAYER_NAME, self.name,
            ))),
        }
    }

    fn flags_layer(&self) -> Result<ObjectLayer, MapReaderError> {
        match self.layer(FLAGS_LAYER_NAME)?.layer_type() {
            LayerType::ObjectLayer(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map {} is not an object layer",
                FLAGS_LAYER_NAME, self.name,
            ))),
        }
    }

    fn interiors(&self) -> Result<Vec<Interior>, MapReaderError> {
        let interiors_image = self.interiors_image()?;
        let mut interiors = vec![];

        for object in self.interiors_zones_layer()?.objects() {
            interiors.push(match object.shape {
                tiled::ObjectShape::Rect { width, height } => Interior::new(
                    object.x,
                    object.y,
                    width,
                    height,
                    interiors_image.width as f32,
                    interiors_image.height as f32,
                ),
                _ => {
                    return Result::Err(MapReaderError::InvalidLayer(format!(
                        "Layer '{}' in map {} contains non Rect shapes, this is not supported now",
                        INTERIORS_ZONES_LAYER_NAME, self.name,
                    )))
                }
            })
        }

        Ok(interiors)
    }

    fn spawn_zones(&self) -> Result<Vec<SpawnZone>, MapReaderError> {
        let background_image = self.background_image()?;
        let mut spawn_zones = vec![];

        for object in self.spawn_zones_layer()?.objects() {
            let spawn_zone_name = SpawnZoneName::from_str(&object.name)?;
            if !spawn_zone_name.allowed_for_zone_object() {
                return Err(MapReaderError::InvalidLayer(format!(
                    "Spawn zone name is not allowed : '{}'",
                    &object.name
                )));
            }

            spawn_zones.push(match object.shape {
                tiled::ObjectShape::Rect { width, height } => SpawnZone::new(
                    spawn_zone_name,
                    object.x,
                    object.y,
                    width,
                    height,
                    background_image.width as f32,
                    background_image.height as f32,
                ),
                _ => {
                    return Result::Err(MapReaderError::InvalidLayer(format!(
                        "Layer '{}' in map {} contains non Rect shapes, this is not supported now",
                        SPAWN_ZONES_LAYER_NAME, self.name,
                    )))
                }
            })
        }

        Ok(spawn_zones)
    }

    fn flags(&self) -> Result<Vec<Flag>, MapReaderError> {
        let mut flags = vec![];

        for object in self.flags_layer()?.objects() {
            let flag_name = FlagName(object.name.clone());

            flags.push(match object.shape {
                tiled::ObjectShape::Rect { width, height } => {
                    Flag::new(flag_name, object.x, object.y, width, height)
                }
                _ => {
                    return Result::Err(MapReaderError::InvalidLayer(format!(
                        "Layer '{}' in map {} contains non Rect shapes, this is not supported now",
                        FLAGS_LAYER_NAME, self.name,
                    )))
                }
            })
        }

        Ok(flags)
    }

    fn terrain_layer(&self) -> Result<FiniteTileLayer, MapReaderError> {
        match self.layer(TERRAIN_LAYER_NAME)?.layer_type() {
            LayerType::TileLayer(layer) => match layer{
                TileLayer::Finite(layer) => Ok(layer),
                TileLayer::Infinite(_) => Result::Err(MapReaderError::InvalidLayer(format!(
                    "Layer '{}' in map {} is an infinite tile layer, but on finite layer is supported",
                    TERRAIN_LAYER_NAME, self.name,
                ))),
            },
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map {} is not an tile layer",
                TERRAIN_LAYER_NAME, self.name,
            ))),
        }
    }

    fn decor_layer(&self) -> Result<FiniteTileLayer, MapReaderError> {
        match self.layer(DECOR_LAYER_NAME)?.layer_type() {
            LayerType::TileLayer(layer) => match layer{
                TileLayer::Finite(layer) => Ok(layer),
                TileLayer::Infinite(_) => Result::Err(MapReaderError::InvalidLayer(format!(
                    "Layer '{}' in map {} is an infinite tile layer, but on finite layer is supported",
                    DECOR_LAYER_NAME, self.name,
                ))),
            },
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map {} is not an tile layer",
                DECOR_LAYER_NAME, self.name,
            ))),
        }
    }

    fn width(&self) -> Result<u32, MapReaderError> {
        Ok(self.terrain_layer()?.width())
    }

    fn height(&self) -> Result<u32, MapReaderError> {
        Ok(self.terrain_layer()?.height())
    }

    fn tile_width(&self) -> Result<u32, MapReaderError> {
        Ok(self.terrain_tileset()?.tile_width)
    }

    fn tile_height(&self) -> Result<u32, MapReaderError> {
        Ok(self.terrain_tileset()?.tile_height)
    }

    fn terrain_tileset(&self) -> Result<&Arc<Tileset>, MapReaderError> {
        match self
            .map
            .tilesets()
            .iter()
            .filter(|tileset| tileset.name == TERRAIN_TILESET_NAME)
            .collect::<Vec<&Arc<Tileset>>>()
            .first()
        {
            Some(tileset) => Ok(tileset),
            None => Result::Err(MapReaderError::TileSetNotFound(format!(
                "Can't found terrain tileset in map {} must exist but is not found",
                self.name,
            ))),
        }
    }

    fn terrain_image(&self) -> Result<Image, MapReaderError> {
        match &self.terrain_tileset()?.image {
            Some(image) => Ok(image.clone()),
            None => Result::Err(MapReaderError::InvalidTileSet(format!(
                "Terrain tileset in map {} should contains image",
                self.name,
            ))),
        }
    }

    fn terrain_tiles(&self) -> Result<Vec<TerrainTile>, MapReaderError> {
        let layer = self.terrain_layer()?;
        let terrain_tileset = self.terrain_tileset()?;
        let terrain_image = self.terrain_image()?;
        let mut tiles = vec![];

        for y in 0..layer.height() {
            for x in 0..layer.width() {
                let layer_tile_data = match layer.get_tile_data(x as i32, y as i32) {
                    Some(layer_tile_data) => layer_tile_data,
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                        "Tile at '{}'x'{}' in terrain layer in map {} must exist but is not found",
                        x, y, self.name,
                    )))
                    }
                };
                let tile_data = match terrain_tileset.get_tile(layer_tile_data.id()) {
                    Some(tile) => tile.clone(),
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile '{}' in terrain layer in map {} is not found in tilesets",
                            layer_tile_data.id(),
                            self.name,
                        )))
                    }
                };

                let id = match tile_data.properties.get(TILE_ID_PROPERTY_KEY) {
                    Some(id) => match id {
                        tiled::PropertyValue::StringValue(id) => id,
                        _ => {
                            return Result::Err(MapReaderError::TileError(format!(
                            "Tile '{}' in terrain layer in map {} should contains {} string property but it is not",
                            layer_tile_data.id(),
                            TILE_ID_PROPERTY_KEY,
                            self.name,
                        )))
                        }
                    },
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile '{}' in terrain layer in map {} should contains {} property",
                            layer_tile_data.id(),
                            TILE_ID_PROPERTY_KEY,
                            self.name,
                        )))
                    }
                };
                let tile_width = terrain_tileset.tile_width;
                let tile_height = terrain_tileset.tile_height;
                let relative_tile_width = tile_width as f32 / terrain_image.width as f32;
                let relative_tile_height = tile_height as f32 / terrain_image.height as f32;

                let tile_id = layer_tile_data.id();
                let tile_y = tile_id / terrain_tileset.columns;
                let tile_x = tile_id - (tile_y * terrain_tileset.columns);

                let terrain_tile = TerrainTile::from_str_id(
                    id,
                    tile_width,
                    tile_height,
                    relative_tile_width,
                    relative_tile_height,
                    x,
                    y,
                    tile_x,
                    tile_y,
                )?;

                tiles.push(terrain_tile)
            }
        }

        Ok(tiles)
    }

    fn decor_tilesets(&self) -> Result<DecorTilesets, MapReaderError> {
        let layer = self.decor_layer()?;
        let mut tileset_indexes = vec![];
        let mut tilesets = vec![];
        let mut positions = HashMap::new();

        for x in 0..layer.width() {
            for y in 0..layer.height() {
                if let Some(layer_tile_data) = layer.get_tile_data(x as i32, y as i32) {
                    if !tileset_indexes.contains(&layer_tile_data.tileset_index()) {
                        tileset_indexes.push(layer_tile_data.tileset_index());
                    }
                };
            }
        }

        for (i, tileset) in self.map.tilesets().iter().enumerate() {
            if tileset_indexes.contains(&i) {
                positions.insert(i, tilesets.len());
                tilesets.push(tileset.clone());
            }
        }

        Ok((tilesets, positions))
    }

    fn decor_images(&self) -> Result<Vec<Image>, MapReaderError> {
        let mut images = vec![];
        let (tilesets, _) = self.decor_tilesets()?;

        for tileset in tilesets {
            match &tileset.image {
                Some(image) => images.push(image.clone()),
                None => {
                    return Result::Err(MapReaderError::InvalidTileSet(format!(
                        "All decor tileset in map {} must contais image",
                        self.name,
                    )))
                }
            };
        }

        Ok(images)
    }

    fn decor(&self) -> Result<Decor, MapReaderError> {
        let decor_layer = self.decor_layer()?;
        let (_, tilesets_positions) = self.decor_tilesets()?;
        let images = self.decor_images()?;
        let image_paths = images
            .iter()
            .map(|image| {
                PathBuf::from("/")
                    .join(
                        image
                            .source
                            .strip_prefix(&self.resources)
                            .expect("Must be in resources"),
                    )
                    .clone()
            })
            .collect();

        let mut tiles = vec![];

        for x in 0..decor_layer.width() {
            for y in 0..decor_layer.height() {
                if let Some(layer_tile_data) = decor_layer.get_tile_data(x as i32, y as i32) {
                    let tileset = self.map.tilesets()[layer_tile_data.tileset_index()].clone();

                    let decor_tileset_position = *tilesets_positions
                        .get(&layer_tile_data.tileset_index())
                        .expect("Positions must are consistent");
                    let image = images
                        .get(decor_tileset_position)
                        .expect("Positions must are consistent");
                    let tile_width = tileset.tile_width;
                    let tile_height = tileset.tile_height;
                    let relative_tile_width = tile_width as f32 / image.width as f32;
                    let relative_tile_height = tile_height as f32 / image.height as f32;

                    let tile_id = layer_tile_data.id();
                    let tile_y = tile_id / tileset.columns;
                    let tile_x = tile_id - (tile_y * tileset.columns);

                    let terrain_tile = DecorTile::new(
                        decor_tileset_position,
                        tile_width,
                        tile_height,
                        relative_tile_width,
                        relative_tile_height,
                        x,
                        y,
                        tile_x,
                        tile_y,
                    );

                    tiles.push(terrain_tile)
                };
            }
        }

        Ok(Decor::new(image_paths, tiles))
    }

    pub fn build(&self) -> Result<Map, MapReaderError> {
        let background_image_path = PathBuf::from("/").join(
            self.background_image()?
                .source
                .strip_prefix(&self.resources)
                .expect("Must be in resources"),
        );
        let interiors_image_path = PathBuf::from("/").join(
            self.interiors_image()?
                .source
                .strip_prefix(&self.resources)
                .expect("Must be in resources"),
        );
        let terrain_image_path = PathBuf::from("/").join(
            self.terrain_image()?
                .source
                .strip_prefix(&self.resources)
                .expect("Must be in resources"),
        );

        let interiors = self.interiors()?;
        let spawn_zones = self.spawn_zones()?;
        let width = self.width()?;
        let height = self.height()?;
        let tile_width = self.tile_width()?;
        let tile_height = self.tile_height()?;
        let terrain_tiles = self.terrain_tiles()?;
        let decor = self.decor()?;
        let flags = self.flags()?;

        Ok(Map::new(
            self.name.clone(),
            background_image_path,
            interiors_image_path,
            terrain_image_path,
            interiors,
            spawn_zones,
            width,
            height,
            terrain_tiles,
            tile_width,
            tile_height,
            decor,
            flags,
        ))
    }
}
