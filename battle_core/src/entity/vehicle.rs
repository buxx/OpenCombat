use crate::{
    config::{TARGET_FPS, VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT},
    deployment::VehicleDeployment,
    graphics::vehicle::VehicleGraphicInfos,
    types::*,
    utils::WorldShape,
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

    pub fn chassis_rotation_speed(&self) -> Angle {
        match self {
            VehicleType::T26 => Angle(0.00125),
        }
    }

    pub fn _main_turret_rotation_speed(&self) -> Angle {
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
    uuid: VehicleIndex,
    type_: VehicleType,
    world_point: WorldPoint,
    chassis_orientation: Angle,
    main_turret_relative_orientation: Angle,
}

impl Vehicle {
    pub fn new(uuid: VehicleIndex, type_: VehicleType, world_point: WorldPoint) -> Self {
        Self {
            uuid,
            type_,
            world_point,
            chassis_orientation: Angle(0.),
            main_turret_relative_orientation: Angle(0.),
        }
    }

    pub fn from_vehicle(vehicle: &Vehicle) -> Self {
        Self {
            uuid: *vehicle.uuid(),
            type_: vehicle.type_().clone(),
            world_point: vehicle.world_point(),
            chassis_orientation: *vehicle.chassis_orientation(),
            main_turret_relative_orientation: *vehicle.main_turret_relative_orientation(),
        }
    }

    pub fn uuid(&self) -> &VehicleIndex {
        &self.uuid
    }

    pub fn world_point(&self) -> WorldPoint {
        self.world_point
    }

    pub fn set_world_point(&mut self, point: WorldPoint) {
        self.world_point = point
    }

    pub fn type_(&self) -> &VehicleType {
        &self.type_
    }

    pub fn chassis_orientation(&self) -> &Angle {
        &self.chassis_orientation
    }

    pub fn chassis_orientation_match(&self, angle: &Angle) -> bool {
        let rounded_chassis_orientation = (self.chassis_orientation().0
            * VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT)
            .round()
            .abs();
        let target_vehicle_orientation = (angle.0
            * VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT)
            .round()
            .abs();

        rounded_chassis_orientation != target_vehicle_orientation
    }

    pub fn main_turret_relative_orientation(&self) -> &Angle {
        &self.main_turret_relative_orientation
    }

    pub fn set_chassis_orientation(&mut self, orientation: Angle) {
        self.chassis_orientation = orientation
    }

    pub fn _set_main_turret_relative_orientation(&mut self, orientation: Angle) {
        self.main_turret_relative_orientation = orientation
    }

    pub fn chassis_shape(&self) -> WorldShape {
        VehicleGraphicInfos::from_type(&self.type_)
            .chassis_physics()
            .from_point(self.world_point)
            .centered()
            .rotate(&self.chassis_orientation)
    }
}

impl From<&VehicleDeployment> for Vehicle {
    fn from(vehicle: &VehicleDeployment) -> Self {
        Self::new(
            vehicle.uuid(),
            vehicle.type_().clone(),
            vehicle.world_point(),
        )
    }
}
