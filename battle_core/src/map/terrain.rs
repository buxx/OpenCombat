use std::fmt::Display;

use crate::{game::posture::Posture, types::Coverage};

#[derive(Clone)]
pub enum TileType {
    ShortGrass,
    MiddleGrass,
    HighGrass,
    Dirt,
    Mud,
    Concrete,
    BrickWall,
    Trunk,
    Water,
    DeepWater,
    Underbrush,
    LightUnderbrush,
    MiddleWoodLogs,
    Hedge,
    MiddleRock,
}

impl TileType {
    pub fn from_str(value: &str) -> Result<Self, TerrainTileError> {
        match value {
            "ShortGrass" => Ok(Self::ShortGrass),
            "MiddleGrass" => Ok(Self::MiddleGrass),
            "HighGrass" => Ok(Self::HighGrass),
            "Dirt" => Ok(Self::Dirt),
            "Mud" => Ok(Self::Mud),
            "Concrete" => Ok(Self::Concrete),
            "BrickWall" => Ok(Self::BrickWall),
            "Trunk" => Ok(Self::Trunk),
            "Water" => Ok(Self::Water),
            "DeepWater" => Ok(Self::DeepWater),
            "Underbrush" => Ok(Self::Underbrush),
            "LightUnderbrush" => Ok(Self::LightUnderbrush),
            "MiddleWoodLogs" => Ok(Self::MiddleWoodLogs),
            "Hedge" => Ok(Self::Hedge),
            "MiddleRock" => Ok(Self::MiddleRock),
            _ => Result::Err(TerrainTileError::UnknownId(value.to_string())),
        }
    }

    pub fn pedestrian_cost(&self) -> i32 {
        match self {
            TileType::ShortGrass => 10,
            TileType::MiddleGrass => 10,
            TileType::HighGrass => 10,
            TileType::Dirt => 11,
            TileType::Mud => 11,
            TileType::Concrete => 50,
            TileType::BrickWall => 50,
            TileType::Trunk => 50,
            TileType::Water => 18,
            TileType::DeepWater => 50,
            TileType::Underbrush => 12,
            TileType::LightUnderbrush => 11,
            TileType::MiddleWoodLogs => 30,
            TileType::Hedge => 20,
            TileType::MiddleRock => 25,
        }
    }

    pub fn block_vehicle(&self) -> bool {
        match self {
            TileType::ShortGrass
            | TileType::MiddleGrass
            | TileType::HighGrass
            | TileType::Dirt
            | TileType::Mud
            | TileType::Concrete
            | TileType::Water
            | TileType::Underbrush
            | TileType::LightUnderbrush
            | TileType::MiddleWoodLogs
            | TileType::Hedge => false,
            TileType::BrickWall | TileType::Trunk | TileType::DeepWater | TileType::MiddleRock => {
                true
            }
        }
    }

    pub fn coverage(&self, posture: &Posture) -> Option<Coverage> {
        match posture {
            Posture::StandUp => match self {
                TileType::ShortGrass => None,
                TileType::MiddleGrass => None,
                TileType::HighGrass => None,
                TileType::Dirt => None,
                TileType::Mud => None,
                TileType::Concrete => None,
                TileType::BrickWall => Some(Coverage(0.8)),
                TileType::Trunk => Some(Coverage(0.9)),
                TileType::Water => None,
                TileType::DeepWater => None,
                TileType::Underbrush => None,
                TileType::LightUnderbrush => None,
                TileType::MiddleWoodLogs => Some(Coverage(0.2)),
                TileType::Hedge => Some(Coverage(0.15)),
                TileType::MiddleRock => Some(Coverage(0.2)),
            },
            Posture::Flat => match self {
                TileType::ShortGrass => None,
                TileType::MiddleGrass => None,
                TileType::HighGrass => None,
                TileType::Dirt => None,
                TileType::Mud => None,
                TileType::Concrete => None,
                TileType::BrickWall => Some(Coverage(0.8)),
                TileType::Trunk => Some(Coverage(0.9)),
                TileType::Water => None,
                TileType::DeepWater => None,
                TileType::Underbrush => None,
                TileType::LightUnderbrush => None,
                TileType::MiddleWoodLogs => Some(Coverage(0.7)),
                TileType::Hedge => Some(Coverage(0.15)),
                TileType::MiddleRock => Some(Coverage(0.9)),
            },
        }
    }
}

#[derive(Debug)]
pub enum TerrainTileError {
    UnknownId(String),
}

impl Display for TerrainTileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerrainTileError::UnknownId(id) => f.write_str(&format!("Unknown id : {}", id)),
        }
    }
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
    ) -> Result<Self, TerrainTileError> {
        Ok(Self {
            type_: TileType::from_str(id)?,
            tile_width,
            tile_height,
            relative_tile_width,
            relative_tile_height,
            x,
            y,
            tile_x,
            tile_y,
        })
    }

    pub fn type_(&self) -> &TileType {
        &self.type_
    }
}
