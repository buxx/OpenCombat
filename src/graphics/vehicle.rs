use crate::{entity::vehicle::OnBoardPlace, graphics::SpriteInfo, types::*};

pub struct VehicleGraphicInfos {
    body: SpriteInfo,
    // Offset is the turret rotation center decal from tank sprite center
    main_turret: Option<(RelativeOffset, SpriteInfo)>,
    // Offset is the place decal from tank sprite center
    places: VehicleGraphicPlaces,
}

impl VehicleGraphicInfos {
    pub fn tank(
        body: SpriteInfo,
        main_turret: (RelativeOffset, SpriteInfo),
        places: Vec<(OnBoardPlace, Offset)>,
    ) -> Self {
        let places = places.into_iter().collect();
        Self {
            body,
            main_turret: Some(main_turret),
            places,
        }
    }

    pub fn body(&self) -> &SpriteInfo {
        &self.body
    }

    pub fn main_turret(&self) -> &Option<(RelativeOffset, SpriteInfo)> {
        &self.main_turret
    }

    pub fn places(&self) -> &VehicleGraphicPlaces {
        &self.places
    }
}
