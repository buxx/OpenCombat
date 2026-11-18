use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    place::{Place, PlaceName},
    spawn::SpawnZoneName,
    tile::Tile,
};
use oc_geo::tile::WorldTileIndex;
use oc_mod::Mod;
use oc_root::{
    DEFAULT_DECOR_DESTRUCTION_RESISTANCE, DEFAULT_INTERIOR_DESTRUCTION_RESISTANCE,
    DEFAULT_INTERIOR_PENETRATION_PROTECT, WorldConfig, geo::WorldVec2,
};
use tiled::{
    FiniteTileLayer, Image, ImageLayer, Layer, LayerType, Loader, Map as TiledMap, ObjectLayer,
    TileLayer, Tileset,
};

use crate::flag::{Flag, FlagName};

use crate::{decor::Decor, interior::Interior, map::Map, spawn::SpawnZone};

const BACKGROUND_IMAGE_LAYER_NAME: &str = "background_image";
const INTERIORS_IMAGE_LAYER_NAME: &str = "interiors_image";
const PENETRATION_PROTECT_KEY: &str = "PENETRATION_PROTECT";
const DESTRUCTION_ALTER_NATURES_KEY: &str = "DESTRUCTION_ALTER_NATURES";
const DESTRUCTION_RESISTANCE_KEY: &str = "DESTRUCTION_RESISTANCE";
const INTERIORS_ZONES_LAYER_NAME: &str = "interiors_zones";
const SPAWN_ZONES_LAYER_NAME: &str = "spawn_zones";
const PLACES_LAYER_NAME: &str = "places";
const FLAGS_LAYER_NAME: &str = "flags";
const DECOR_LAYER_NAME: &str = "decor";
const TERRAIN_LAYER_NAME: &str = "terrain";
const HEIGHT_LAYER_NAME: &str = "height";
const TERRAIN_TILESET_NAME: &str = "terrain";
const DECOR_TILESET_NAME: &str = "decor";
const TILE_ID_PROPERTY_KEY: &str = "ID";

#[derive(Debug, thiserror::Error)]
pub enum MapReaderError {
    #[error("Map not found: {0}")]
    MapNotFound(String),
    #[error("Layer not found: {0}")]
    LayerNotFound(String),
    #[error("Invalid layer: {0}")]
    InvalidLayer(String),
    #[error("Tile set not found: {0}")]
    TileSetNotFound(String),
    #[error("Invalid tile set: {0}")]
    InvalidTileSet(String),
    #[error("Tile error: {0}")]
    TileError(String),
    #[error("Terrain tile error: {0}")]
    TerrainTileError(String),
}

#[derive(Debug, Clone)]
pub struct MapReader {
    folder: PathBuf,
    map: TiledMap,
}

impl MapReader {
    pub fn new(path: &Path) -> Result<Self, MapReaderError> {
        let mut loader = Loader::new();

        let map = match loader.load_tmx_map(path.join("world.tmx")) {
            Ok(map) => map,
            Err(error) => {
                return Result::Err(MapReaderError::MapNotFound(format!(
                    "Failed to load map {} : {}",
                    path.display(),
                    error
                )));
            }
        };

        Ok(Self {
            folder: path.to_path_buf(),
            map,
        })
    }

    fn layer(&self, name: &str) -> Result<Layer<'_>, MapReaderError> {
        match self
            .map
            .layers()
            .filter(|layer| layer.name == name)
            .collect::<Vec<Layer>>()
            .first()
        {
            Some(layer) => Ok(*layer),
            None => Result::Err(MapReaderError::LayerNotFound(format!(
                "Failed to find layer '{}' in map",
                name,
            ))),
        }
    }

    fn background_image_layer(&self) -> Result<ImageLayer<'_>, MapReaderError> {
        match self.layer(BACKGROUND_IMAGE_LAYER_NAME)?.layer_type() {
            LayerType::Image(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an image layer",
                BACKGROUND_IMAGE_LAYER_NAME,
            ))),
        }
    }

    fn background_image(&self) -> Result<Image, MapReaderError> {
        match &self.background_image_layer()?.image {
            Some(image) => Ok(image.clone()),
            None => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in ma must contains image",
                BACKGROUND_IMAGE_LAYER_NAME,
            ))),
        }
    }

    fn interiors_image_layer(&self) -> Result<ImageLayer<'_>, MapReaderError> {
        match self.layer(INTERIORS_IMAGE_LAYER_NAME)?.layer_type() {
            LayerType::Image(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an image layer",
                INTERIORS_IMAGE_LAYER_NAME,
            ))),
        }
    }

    fn interiors_image(&self) -> Result<Image, MapReaderError> {
        match &self.interiors_image_layer()?.image {
            Some(image) => Ok(image.clone()),
            None => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map must contains image",
                INTERIORS_IMAGE_LAYER_NAME,
            ))),
        }
    }

    fn interiors_zones_layer(&self) -> Result<ObjectLayer<'_>, MapReaderError> {
        match self.layer(INTERIORS_ZONES_LAYER_NAME)?.layer_type() {
            LayerType::Objects(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an object layer",
                INTERIORS_ZONES_LAYER_NAME,
            ))),
        }
    }

    fn spawn_zones_layer(&self) -> Result<ObjectLayer<'_>, MapReaderError> {
        match self.layer(SPAWN_ZONES_LAYER_NAME)?.layer_type() {
            LayerType::Objects(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an object layer",
                SPAWN_ZONES_LAYER_NAME,
            ))),
        }
    }

    fn places_layer(&self) -> Result<ObjectLayer<'_>, MapReaderError> {
        match self.layer(PLACES_LAYER_NAME)?.layer_type() {
            LayerType::Objects(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an object layer",
                SPAWN_ZONES_LAYER_NAME,
            ))),
        }
    }

    fn flags_layer(&self) -> Result<ObjectLayer<'_>, MapReaderError> {
        match self.layer(FLAGS_LAYER_NAME)?.layer_type() {
            LayerType::Objects(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an object layer",
                FLAGS_LAYER_NAME,
            ))),
        }
    }

    pub fn interiors(&self) -> Result<Vec<Interior>, MapReaderError> {
        let mut interiors = vec![];

        for object in self.interiors_zones_layer()?.objects() {
            interiors.push({
                let (x, y) = (object.x, object.y);
                let points: Vec<WorldVec2> = match &object.shape {
                    tiled::ObjectShape::Polygon { points } => points
                        .iter()
                        .map(|(px, py)| WorldVec2::new(x + px, y + py))
                        .collect(),
                    tiled::ObjectShape::Rect { width, height } => vec![
                        WorldVec2::new(x, y),
                        WorldVec2::new(x + width, y),
                        WorldVec2::new(x + width, y + height),
                        WorldVec2::new(x, y + height),
                    ],
                    _ => {
                        return Result::Err(MapReaderError::InvalidLayer(format!(
                            "Layer '{}' in map contains non Rect/Polygon shapes, this is not supported now",
                            INTERIORS_ZONES_LAYER_NAME,
                        )));
                    }
                };
                {
                    // Warning: image crop must fit with this start point to be correctly displayed
                    let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                    let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                    let start = WorldVec2::new(min_x, min_y);

                    let penetration_protect = match object.properties.get(PENETRATION_PROTECT_KEY) {
                        Some(tiled::PropertyValue::FloatValue(v)) => *v,
                        Some(tiled::PropertyValue::IntValue(v)) => *v as f32,
                        _ => DEFAULT_INTERIOR_PENETRATION_PROTECT,
                    };

                    let destruction_resistance =
                        match object.properties.get(DESTRUCTION_RESISTANCE_KEY) {
                            Some(tiled::PropertyValue::FloatValue(v)) => *v,
                            Some(tiled::PropertyValue::IntValue(v)) => *v as f32,
                            _ => DEFAULT_INTERIOR_DESTRUCTION_RESISTANCE,
                        };

                    // Comma separated names
                    let destruction_alter_natures =
                        match object.properties.get(DESTRUCTION_ALTER_NATURES_KEY) {
                            Some(tiled::PropertyValue::StringValue(v)) => Some(
                                v.split(',')
                                    .map(str::trim)
                                    .filter(|name| !name.is_empty())
                                    .map(String::from)
                                    .collect(),
                            ),
                            _ => None,
                        };

                    Interior::new(
                        object.id(),
                        start,
                        points,
                        penetration_protect,
                        destruction_resistance,
                        destruction_alter_natures,
                    )
                }
            });
        }

        Ok(interiors)
    }

    pub fn spawn_zones(&self) -> Result<Vec<SpawnZone>, MapReaderError> {
        let mut spawn_zones = vec![];

        for object in self.spawn_zones_layer()?.objects() {
            let name = SpawnZoneName(object.name.to_string());
            let start = WorldVec2::new(object.x, object.y);

            spawn_zones.push(match &object.shape {
                tiled::ObjectShape::Rect { width, height } => {
                    let (x, y) = (start.x, start.y);
                    let points = vec![
                        WorldVec2::new(x, y),
                        WorldVec2::new(x + width, y),
                        WorldVec2::new(x + width, y + height),
                        WorldVec2::new(x, y + height),
                    ];

                    SpawnZone {
                        name,
                        start,
                        points,
                    }
                }
                tiled::ObjectShape::Polygon { points } => {
                    let points: Vec<WorldVec2> = points
                        .iter()
                        .map(|(x, y)| WorldVec2::new(start.x + x, start.y + y))
                        .collect();

                    SpawnZone {
                        name,
                        start,
                        points,
                    }
                }
                _ => {
                    return Result::Err(MapReaderError::InvalidLayer(format!(
                        "Layer '{}' in map contains non Rect/Polygon shapes, this is not supported now",
                        SPAWN_ZONES_LAYER_NAME,
                    )));
                }
            })
        }

        Ok(spawn_zones)
    }

    pub fn places(&self) -> Result<Vec<Place>, MapReaderError> {
        let mut places = vec![];

        for object in self.places_layer()?.objects() {
            let name = PlaceName(if object.name.trim().is_empty() {
                object.id().to_string()
            } else {
                object.name.clone()
            });

            places.push(match object.shape {
                tiled::ObjectShape::Point(..) => Place::new(name, object.x, object.y),
                _ => {
                    return Result::Err(MapReaderError::InvalidLayer(format!(
                        "Layer '{}' in map contains non point shape, this is not supported now",
                        PLACES_LAYER_NAME,
                    )));
                }
            })
        }

        Ok(places)
    }

    pub fn flags(&self) -> Result<Vec<Flag>, MapReaderError> {
        let mut flags = vec![];

        for object in self.flags_layer()?.objects() {
            let flag_name = FlagName(object.name.clone());

            flags.push(match object.shape {
                tiled::ObjectShape::Rect { width, height } => {
                    let start = WorldVec2::new(object.x, object.y);
                    Flag::new(flag_name, start, width, height)
                }
                _ => {
                    return Result::Err(MapReaderError::InvalidLayer(format!(
                        "Layer '{}' in map contains non Rect shapes, this is not supported now",
                        FLAGS_LAYER_NAME,
                    )));
                }
            })
        }

        Ok(flags)
    }

    fn terrain_layer(&self) -> Result<FiniteTileLayer<'_>, MapReaderError> {
        match self.layer(TERRAIN_LAYER_NAME)?.layer_type() {
            LayerType::Tiles(layer) => match layer {
                TileLayer::Finite(layer) => Ok(layer),
                TileLayer::Infinite(_) => Result::Err(MapReaderError::InvalidLayer(format!(
                    "Layer '{}' in map is an infinite tile layer, but on finite layer is supported",
                    TERRAIN_LAYER_NAME,
                ))),
            },
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an tile layer",
                TERRAIN_LAYER_NAME,
            ))),
        }
    }

    fn height_layer(&self) -> Result<FiniteTileLayer<'_>, MapReaderError> {
        match self.layer(HEIGHT_LAYER_NAME)?.layer_type() {
            LayerType::Tiles(layer) => match layer {
                TileLayer::Finite(layer) => Ok(layer),
                TileLayer::Infinite(_) => Result::Err(MapReaderError::InvalidLayer(format!(
                    "Layer '{}' in map is an infinite tile layer, but on finite layer is supported",
                    HEIGHT_LAYER_NAME,
                ))),
            },
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an tile layer",
                HEIGHT_LAYER_NAME,
            ))),
        }
    }

    fn decor_layer(&self) -> Result<(Layer<'_>, FiniteTileLayer<'_>), MapReaderError> {
        let decor_layer = self.layer(DECOR_LAYER_NAME)?;
        match decor_layer.layer_type() {
            LayerType::Tiles(layer) => match layer {
                TileLayer::Finite(layer_) => Ok((decor_layer, layer_)),
                TileLayer::Infinite(_) => Result::Err(MapReaderError::InvalidLayer(format!(
                    "Layer '{}' in map is an infinite tile layer, but on finite layer is supported",
                    DECOR_LAYER_NAME,
                ))),
            },
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an tile layer",
                DECOR_LAYER_NAME,
            ))),
        }
    }

    pub fn width(&self) -> Result<u32, MapReaderError> {
        Ok(self.terrain_layer()?.width())
    }

    pub fn height(&self) -> Result<u32, MapReaderError> {
        Ok(self.terrain_layer()?.height())
    }

    pub fn size_pixels(&self) -> (f32, f32) {
        (
            (self.map.width * self.map.tile_width) as f32,
            (self.map.height * self.map.tile_height) as f32,
        )
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
            None => Result::Err(MapReaderError::TileSetNotFound(
                "Can't found terrain tileset in map must exist but is not found".to_string(),
            )),
        }
    }

    fn terrain_image(&self) -> Result<Image, MapReaderError> {
        match &self.terrain_tileset()?.image {
            Some(image) => Ok(image.clone()),
            None => Result::Err(MapReaderError::InvalidTileSet(
                "Terrain tileset in map should contains image".to_string(),
            )),
        }
    }

    pub fn tiles(&self, _: &WorldConfig, mod_: &Mod) -> Result<Vec<Tile>, MapReaderError> {
        let terrain_layer = self.terrain_layer()?;
        let height_layer = self.height_layer()?;
        let terrain_tileset = self.terrain_tileset()?;
        let mut tiles = vec![];

        let mut tile_id: u64 = 0;
        for y in 0..terrain_layer.height() {
            for x in 0..terrain_layer.width() {
                let terrain_layer_tile_data = match terrain_layer.get_tile_data(x as i32, y as i32)
                {
                    Some(data) => data,
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile at '{}'x'{}' in terrain layer in map must exist but is not found",
                            x, y,
                        )));
                    }
                };
                let height_layer_tile_data = match height_layer.get_tile_data(x as i32, y as i32) {
                    Some(data) => data,
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile at '{}'x'{}' in height layer in map must exist but is not found",
                            x, y,
                        )));
                    }
                };
                let terrain_tile_data = match terrain_tileset.get_tile(terrain_layer_tile_data.id())
                {
                    Some(tile) => tile.clone(),
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile '{}' in terrain layer in map is not found in tilesets",
                            terrain_layer_tile_data.id(),
                        )));
                    }
                };

                let id = match terrain_tile_data.properties.get(TILE_ID_PROPERTY_KEY) {
                    Some(id) => match id {
                        tiled::PropertyValue::StringValue(id) => id,
                        _ => {
                            return Result::Err(MapReaderError::TileError(format!(
                                "Tile '{}' in terrain layer in map should contains {} string property but it is not",
                                terrain_layer_tile_data.id(),
                                TILE_ID_PROPERTY_KEY,
                            )));
                        }
                    },
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile '{}' in terrain layer in map should contains {} property",
                            terrain_layer_tile_data.id(),
                            TILE_ID_PROPERTY_KEY,
                        )));
                    }
                };

                let z = height_layer_tile_data.id() as u8;
                let nature = mod_
                    .nature_index(id)
                    .ok_or(MapReaderError::TerrainTileError(format!(
                        "Terrain nature ID '{id}' unknown from mod"
                    )))?;
                let i = WorldTileIndex(tile_id);
                let prohibe = mod_.nature(nature).traversability.clone();
                let tile = Tile {
                    i,
                    nature,
                    z,
                    allow: prohibe,
                };

                tiles.push(tile);
                tile_id += 1;
            }
        }
        Ok(tiles)
    }

    fn decor_tileset(&self) -> Result<(usize, &Arc<Tileset>), MapReaderError> {
        self.map
            .tilesets()
            .iter()
            .enumerate()
            .find(|(_, tileset)| tileset.name == DECOR_TILESET_NAME)
            .ok_or(MapReaderError::TileSetNotFound(
                "Can't found decor tileset in map must exist but is not found".to_string(),
            ))
    }

    /// Decor of each tile of the map (same order than `tiles`)
    pub fn decor(&self) -> Result<Vec<Option<Decor>>, MapReaderError> {
        let (_, decor_layer) = self.decor_layer()?;
        let (decor_tileset_index, decor_tileset) = self.decor_tileset()?;
        let mut decor_tiles = vec![];

        let mut tile_id: u64 = 0;
        for y in 0..decor_layer.height() {
            for x in 0..decor_layer.width() {
                let tile = match decor_layer.get_tile_data(x as i32, y as i32) {
                    Some(data) if data.tileset_index() == decor_tileset_index => {
                        let tile_data = decor_tileset.get_tile(data.id());
                        let property = tile_data
                            .as_ref()
                            .and_then(|tile| tile.properties.get(DESTRUCTION_RESISTANCE_KEY));
                        let destruction_resistance = match property {
                            Some(tiled::PropertyValue::FloatValue(v)) => *v,
                            Some(tiled::PropertyValue::IntValue(v)) => *v as f32,
                            _ => DEFAULT_DECOR_DESTRUCTION_RESISTANCE,
                        };
                        Some(Decor::new(
                            WorldTileIndex(tile_id),
                            data.id(),
                            destruction_resistance,
                        ))
                    }
                    Some(data) => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile at '{}'x'{}' in decor layer in map must be from '{}' tileset (found tileset {})",
                            x,
                            y,
                            DECOR_TILESET_NAME,
                            data.tileset_index(),
                        )));
                    }
                    None => None,
                };
                decor_tiles.push(tile);
                tile_id += 1;
            }
        }

        Ok(decor_tiles)
    }

    pub fn build(&self) -> Result<Map, MapReaderError> {
        let background_image_path = self.folder.join(&self.background_image()?.source);
        let interiors_image_path = self.folder.join(&self.interiors_image()?.source);
        let terrain_image_path = self.folder.join(&self.terrain_image()?.source);

        let interiors = self.interiors()?;
        let spawn_zones = self.spawn_zones()?;
        let width = self.width()?;
        let height = self.height()?;
        let decor = self.decor()?;
        let flags = self.flags()?;
        let places = self.places()?;

        Ok(Map::new(
            background_image_path,
            interiors_image_path,
            terrain_image_path,
            interiors,
            spawn_zones,
            width,
            height,
            decor,
            flags,
            places,
        ))
    }
}
