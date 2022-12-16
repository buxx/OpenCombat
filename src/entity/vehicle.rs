use crate::{
    config::TARGET_FPS,
    graphics::{vehicle::VehicleGraphicInfos, SpriteInfo},
    types::*,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum VehicleType {
    T26,
}

impl VehicleType {
    pub fn board_composition(&self) -> BoardComposition {
        match self {
            VehicleType::T26 => vec![OnBoardPlace::Driver, OnBoardPlace::MainTurretGunner],
        }
    }

    pub fn rotation_speed(&self) -> Angle {
        match self {
            VehicleType::T26 => Angle(0.00125),
        }
    }

    pub fn drive_speed(&self) -> f32 {
        match self {
            VehicleType::T26 => 5.0 / TARGET_FPS as f32,
        }
    }

    pub fn sprites_infos(&self) -> VehicleGraphicInfos {
        match self {
            VehicleType::T26 => {
                VehicleGraphicInfos::tank(
                    SpriteInfo::new(0., 192., 64., 96.),
                    (
                        RelativeOffset::new(0.05, 0.),
                        SpriteInfo::new(128., 192., 64., 96.),
                    ),
                    // FIXME BS NOW : These positions must strictly match with board_composition (check it at startup ?)
                    vec![
                        (OnBoardPlace::Driver, Offset::new(8., -16.)),
                        (OnBoardPlace::MainTurretGunner, Offset::new(-3., 0.)),
                    ],
                )
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq, Hash)]
pub enum OnBoardPlace {
    Driver,
    MainTurretGunner,
    MainCommandment,
    Passenger1,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Vehicle {
    type_: VehicleType,
    world_point: WorldPoint,
    orientation: Angle,
}

impl Vehicle {
    pub fn new(type_: VehicleType, world_point: WorldPoint) -> Self {
        Self {
            type_,
            world_point,
            orientation: Angle(0.),
        }
    }

    pub fn get_world_point(&self) -> WorldPoint {
        self.world_point
    }

    pub fn set_world_point(&mut self, point: WorldPoint) {
        self.world_point = point
    }

    pub fn get_type(&self) -> &VehicleType {
        &self.type_
    }

    pub fn get_orientation(&self) -> &Angle {
        &self.orientation
    }

    pub fn set_orientation(&mut self, orientation: Angle) {
        self.orientation = orientation
    }
}
