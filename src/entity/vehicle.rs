use crate::{config::TARGET_FPS, types::*};
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

    pub fn chassis_rotation_speed(&self) -> Angle {
        match self {
            VehicleType::T26 => Angle(0.00125),
        }
    }

    pub fn main_turret_rotation_speed(&self) -> Angle {
        match self {
            VehicleType::T26 => Angle(0.00255),
        }
    }

    pub fn drive_speed(&self) -> f32 {
        match self {
            VehicleType::T26 => 5.0 / TARGET_FPS as f32,
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
    chassis_orientation: Angle,
    main_turret_relative_orientation: Angle,
}

impl Vehicle {
    pub fn new(type_: VehicleType, world_point: WorldPoint) -> Self {
        Self {
            type_,
            world_point,
            chassis_orientation: Angle(0.),
            main_turret_relative_orientation: Angle(0.),
        }
    }

    pub fn from_vehicle(vehicle: &Vehicle) -> Self {
        Self {
            type_: vehicle.get_type().clone(),
            world_point: vehicle.get_world_point(),
            chassis_orientation: vehicle.get_chassis_orientation().clone(),
            main_turret_relative_orientation: vehicle
                .get_main_turret_relative_orientation()
                .clone(),
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

    pub fn get_chassis_orientation(&self) -> &Angle {
        &self.chassis_orientation
    }

    pub fn get_main_turret_relative_orientation(&self) -> &Angle {
        &self.main_turret_relative_orientation
    }

    pub fn set_chassis_orientation(&mut self, orientation: Angle) {
        self.chassis_orientation = orientation
    }

    pub fn set_main_turret_relative_orientation(&mut self, orientation: Angle) {
        self.main_turret_relative_orientation = orientation
    }
}
