use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DebugTerrain {
    None,
    Tiles,
    Opacity,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DebugPhysics {
    None,
    x762x54BulletFire,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DebugLevel {
    Debug0,
    Debug1,
    Debug2,
    Debug3,
}

impl DebugLevel {
    pub fn scene_item_circles(&self) -> bool {
        match self {
            DebugLevel::Debug0 => false,
            DebugLevel::Debug1 => true,
            DebugLevel::Debug2 => true,
            DebugLevel::Debug3 => true,
        }
    }
    pub fn mouse(&self) -> bool {
        match self {
            DebugLevel::Debug0 => false,
            DebugLevel::Debug1 => true,
            DebugLevel::Debug2 => true,
            DebugLevel::Debug3 => true,
        }
    }
    pub fn areas(&self) -> bool {
        match self {
            DebugLevel::Debug0 => false,
            DebugLevel::Debug1 => true,
            DebugLevel::Debug2 => true,
            DebugLevel::Debug3 => true,
        }
    }
    pub fn formation_positions(&self) -> bool {
        match self {
            DebugLevel::Debug0 => false,
            DebugLevel::Debug1 => false,
            DebugLevel::Debug2 => false,
            DebugLevel::Debug3 => true,
        }
    }
    pub fn visibilities(&self) -> bool {
        match self {
            DebugLevel::Debug0 => false,
            DebugLevel::Debug1 => false,
            DebugLevel::Debug2 => true,
            DebugLevel::Debug3 => true,
        }
    }
    // TODO : import code from v1
    // pub fn scene_items_text_infos(&self) -> bool {
    //     match self {
    //         DebugLevel::Debug0 => false,
    //         DebugLevel::Debug1 => false,
    //         DebugLevel::Debug2 => false,
    //         DebugLevel::Debug3 => true,
    //     }
    // }
    pub fn move_paths(&self) -> bool {
        match self {
            DebugLevel::Debug0 => false,
            DebugLevel::Debug1 => false,
            DebugLevel::Debug2 => false,
            DebugLevel::Debug3 => true,
        }
    }
}
