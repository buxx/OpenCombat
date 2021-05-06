pub enum TileId {
    ShortGrass,
    MiddleGrass,
    HighGrass,
    Dirt,
    Mud,
    Concrete,
    BrickWall,
}

pub struct Tile {
    pub id: TileId,
    pub tile_width: u32,
    pub tile_height: u32,
    pub relative_tile_width: f32,
    pub relative_tile_height: f32,
    pub tile_x: u32,
    pub tile_y: u32,
    pub opacity: f32,
}

impl Tile {
    pub fn from_str_id(
        id: &str,
        tile_width: u32,
        tile_height: u32,
        relative_tile_width: f32,
        relative_tile_height: f32,
        tile_x: u32,
        tile_y: u32,
    ) -> Self {
        match id {
            "ShortGrass" => Self {
                id: TileId::ShortGrass,
                opacity: 0.0,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                tile_x,
                tile_y,
            },
            "MiddleGrass" => Self {
                id: TileId::MiddleGrass,
                opacity: 0.1,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                tile_x,
                tile_y,
            },
            "HighGrass" => Self {
                id: TileId::HighGrass,
                opacity: 0.2,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                tile_x,
                tile_y,
            },
            "Dirt" => Self {
                id: TileId::Dirt,
                opacity: 0.0,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                tile_x,
                tile_y,
            },
            "Mud" => Self {
                id: TileId::Mud,
                opacity: 0.1,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                tile_x,
                tile_y,
            },
            "Concrete" => Self {
                id: TileId::Concrete,
                opacity: 0.0,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                tile_x,
                tile_y,
            },
            "BrickWall" => Self {
                id: TileId::BrickWall,
                opacity: 1.0,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                tile_x,
                tile_y,
            },
            &_ => {
                // FIXME BS NOW: manage errors
                panic!("Unknown tile id {}", id)
            }
        }
    }
}
