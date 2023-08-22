use std::path::PathBuf;

#[derive(Clone)]
pub struct DecorTile {
    pub tileset_i: usize, // Used to rely tileset/sprite_batch in Decor
    pub tile_width: u32,
    pub tile_height: u32,
    pub relative_tile_width: f32,
    pub relative_tile_height: f32,
    pub x: u32,
    pub y: u32,
    pub tile_x: u32,
    pub tile_y: u32,
}

impl DecorTile {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tileset_i: usize, // Used to rely tileset/sprite_batch in Decor
        tile_width: u32,
        tile_height: u32,
        relative_tile_width: f32,
        relative_tile_height: f32,
        x: u32,
        y: u32,
        tile_x: u32,
        tile_y: u32,
    ) -> Self {
        Self {
            tileset_i,
            tile_width,
            tile_height,
            relative_tile_width,
            relative_tile_height,
            x,
            y,
            tile_x,
            tile_y,
        }
    }
}

#[derive(Clone)]
pub struct Decor {
    image_paths: Vec<PathBuf>,
    tiles: Vec<DecorTile>,
}

impl Decor {
    pub fn new(image_paths: Vec<PathBuf>, tiles: Vec<DecorTile>) -> Self {
        Self { image_paths, tiles }
    }

    pub fn image_paths(&self) -> &Vec<PathBuf> {
        &self.image_paths
    }

    pub fn tiles(&self) -> &Vec<DecorTile> {
        &self.tiles
    }
}
