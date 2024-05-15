use std::path::Path;

use battle_core::{
    map::{decor::Decor, terrain::TileType, Map},
    types::{GridPoint, Offset},
};

use super::MapModel;

pub struct MapGenerator<T: MapModel> {
    model: T,
    width: u32,
    height: u32,
    default_tile_type: TileType,
    placed: Vec<(GridPoint, TileType)>,
}

impl<T: MapModel> MapGenerator<T> {
    pub fn new(model: T) -> Self {
        Self {
            model,
            width: Default::default(),
            height: Default::default(),
            default_tile_type: TileType::ShortGrass,
            placed: vec![],
        }
    }

    pub fn width(mut self, value: u32) -> Self {
        self.width = value;
        self
    }

    pub fn height(mut self, value: u32) -> Self {
        self.height = value;
        self
    }

    pub fn place(mut self, value: Vec<(GridPoint, TileType)>) -> Self {
        self.placed.extend(value);
        self
    }

    pub fn default_tile_type(mut self, value: TileType) -> Self {
        self.default_tile_type = value;
        self
    }

    pub fn generate(&self) -> Map {
        let terrain_tiles = self.model.terrain_tiles(
            self.width,
            self.height,
            self.default_tile_type.clone(),
            &self.placed,
        );
        let terrain_tile_size = self.model.terrain_tile_size();

        Map::new(
            "example_template".into(),
            Path::new("/maps/example_template/1_white_pixel.png").into(),
            Path::new("/maps/example_template/1_white_pixel.png").into(),
            Path::new("/maps/example_template/1_white_pixel.png").into(),
            vec![],
            vec![],
            self.width / terrain_tile_size,
            self.height / terrain_tile_size,
            terrain_tiles,
            terrain_tile_size,
            terrain_tile_size,
            Decor::new(vec![], vec![], Offset::new(0., 0.)),
            vec![],
        )
    }
}
