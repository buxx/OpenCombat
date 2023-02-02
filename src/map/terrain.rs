use ggez::{GameError, GameResult};

#[derive(Clone)]
pub enum TileType {
    ShortGrass,
    MiddleGrass,
    HighGrass,
    Dirt,
    Mud,
    Concrete,
    BrickWall,
}

#[derive(Clone)]
pub struct TerrainTile {
    pub type_: TileType,
    pub tile_width: u32,
    pub tile_height: u32,
    pub relative_tile_width: f32,
    pub relative_tile_height: f32,
    pub x: u32,
    pub y: u32,
    pub tile_x: u32,
    pub tile_y: u32,
    pub pedestrian_cost: i32,
    pub block_vehicle: bool,
}

impl TerrainTile {
    pub fn from_str_id(
        id: &str,
        tile_width: u32,
        tile_height: u32,
        relative_tile_width: f32,
        relative_tile_height: f32,
        x: u32,
        y: u32,
        tile_x: u32,
        tile_y: u32,
    ) -> GameResult<Self> {
        GameResult::Ok(match id {
            "ShortGrass" => Self {
                type_: TileType::ShortGrass,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                x,
                y,
                tile_x,
                tile_y,
                pedestrian_cost: 10,
                block_vehicle: false,
            },
            "MiddleGrass" => Self {
                type_: TileType::MiddleGrass,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                x,
                y,
                tile_x,
                tile_y,
                pedestrian_cost: 10,
                block_vehicle: false,
            },
            "HighGrass" => Self {
                type_: TileType::HighGrass,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                x,
                y,
                tile_x,
                tile_y,
                pedestrian_cost: 10,
                block_vehicle: false,
            },
            "Dirt" => Self {
                type_: TileType::Dirt,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                x,
                y,
                tile_x,
                tile_y,
                pedestrian_cost: 11,
                block_vehicle: false,
            },
            "Mud" => Self {
                type_: TileType::Mud,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                x,
                y,
                tile_x,
                tile_y,
                pedestrian_cost: 11,
                block_vehicle: false,
            },
            "Concrete" => Self {
                type_: TileType::Concrete,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                x,
                y,
                tile_x,
                tile_y,
                pedestrian_cost: 50,
                block_vehicle: true,
            },
            "BrickWall" => Self {
                type_: TileType::BrickWall,
                tile_width,
                tile_height,
                relative_tile_width,
                relative_tile_height,
                x,
                y,
                tile_x,
                tile_y,
                pedestrian_cost: 50,
                block_vehicle: true,
            },
            &_ => {
                return GameResult::Err(GameError::ResourceLoadError(format!(
                    "Unknown tile '{}'",
                    id
                )))
            }
        })
    }
}
