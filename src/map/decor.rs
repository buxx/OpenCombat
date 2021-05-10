use crate::map::util::extract_image_from_tileset;
use ggez::graphics;
use std::collections::HashMap;
use std::path::Path;
use tiled::{Image as TiledImage, Layer, Tileset};

pub struct DecorTile {
    pub tileset_i: usize, // Used to rely tileset/sprite_batch in Decor
    pub tile_width: u32,
    pub tile_height: u32,
    pub relative_tile_width: f32,
    pub relative_tile_height: f32,
    pub tile_x: u32,
    pub tile_y: u32,
}

impl DecorTile {
    pub fn new(
        tileset_i: usize, // Used to rely tileset/sprite_batch in Decor
        tile_width: u32,
        tile_height: u32,
        relative_tile_width: f32,
        relative_tile_height: f32,
        tile_x: u32,
        tile_y: u32,
    ) -> Self {
        Self {
            tileset_i,
            tile_width,
            tile_height,
            relative_tile_width,
            relative_tile_height,
            tile_x,
            tile_y,
        }
    }
}

pub struct Decor {
    pub layer: Layer,
    pub tilesets: Vec<Tileset>,
    pub images: Vec<TiledImage>,
    // FIXME (u32, u32) -> GridPoint
    pub tiles: HashMap<(u32, u32), DecorTile>,
}

impl Decor {
    pub fn new(
        layer: Layer,
        tilesets: Vec<Tileset>,
        images: Vec<TiledImage>,
        tiles: HashMap<(u32, u32), DecorTile>,
    ) -> Self {
        Self {
            layer,
            tilesets,
            images,
            tiles,
        }
    }
}
