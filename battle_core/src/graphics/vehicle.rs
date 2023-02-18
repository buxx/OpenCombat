use crate::{
    entity::vehicle::{OnBoardPlace, VehicleType},
    types::*,
    utils::WorldShape,
};

use super::SpriteInfo;

const SPRITE_SHEET_WIDTH: f32 = 256.;
const SPRITE_SHEET_HEIGHT: f32 = 96.;
pub const TILE_WIDTH: f32 = 64.;
pub const TILE_HEIGHT: f32 = 96.;

pub struct VehicleGraphicInfos {
    chassis: SpriteInfo,
    // Offset is the turret rotation center decal from tank sprite center
    main_turret: Option<(RelativeOffset, SpriteInfo)>,
    // Offset is the place decal from tank sprite center
    places: VehicleGraphicPlaces,
    /// Used for collisions
    size: VehicleSize,
    /// Chassis physics space
    chassis_physics: WorldShape,
}

impl VehicleGraphicInfos {
    pub fn tank(
        chassis: SpriteInfo,
        main_turret: (RelativeOffset, SpriteInfo),
        places: Vec<(OnBoardPlace, Offset)>,
        size: VehicleSize,
        chassis_physics: WorldShape,
    ) -> Self {
        let places = places.into_iter().collect();
        Self {
            chassis,
            main_turret: Some(main_turret),
            places,
            size,
            chassis_physics,
        }
    }

    pub fn from_type(type_: &VehicleType) -> VehicleGraphicInfos {
        match type_ {
            VehicleType::T26 => {
                VehicleGraphicInfos::tank(
                    SpriteInfo::new(
                        0.,
                        0.,
                        TILE_WIDTH,
                        TILE_HEIGHT,
                        SPRITE_SHEET_WIDTH,
                        SPRITE_SHEET_HEIGHT,
                    ),
                    (
                        RelativeOffset::new(0.05, 0.),
                        SpriteInfo::new(
                            128.,
                            0.,
                            TILE_WIDTH,
                            TILE_HEIGHT,
                            SPRITE_SHEET_WIDTH,
                            SPRITE_SHEET_HEIGHT,
                        ),
                    ),
                    // FIXME BS NOW : These positions must strictly match with board_composition (check it at startup ?)
                    vec![
                        (OnBoardPlace::Driver, Offset::new(8., -16.)),
                        (OnBoardPlace::MainTurretGunner, Offset::new(-3., 0.)),
                    ],
                    // TODO : compute this value according to map grid size (meters)
                    VehicleSize(10),
                    WorldShape::from_meters(Meters(11.), Meters(21.)),
                )
            }
        }
    }

    pub fn chassis(&self) -> &SpriteInfo {
        &self.chassis
    }

    pub fn main_turret(&self) -> &Option<(RelativeOffset, SpriteInfo)> {
        &self.main_turret
    }

    pub fn places(&self) -> &VehicleGraphicPlaces {
        &self.places
    }

    pub fn size(&self) -> &VehicleSize {
        &self.size
    }

    pub fn chassis_physics(&self) -> &WorldShape {
        &self.chassis_physics
    }
}
