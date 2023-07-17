use oc_core::spawn::SpawnZoneName;
use serde::{Deserialize, Serialize};

use crate::{map::Map, types::WorldPoint, utils::WorldShape};

use super::control::MapControl;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct FlagName(pub String);

#[derive(Clone)]
pub struct Flag {
    name: FlagName,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Flag {
    pub fn new(name: FlagName, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            name,
            x,
            y,
            width,
            height,
        }
    }

    pub fn name(&self) -> &FlagName {
        &self.name
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn shape(&self) -> WorldShape {
        WorldShape {
            top_left: WorldPoint::new(self.x, self.y),
            top_right: WorldPoint::new(self.x + self.width, self.y),
            bottom_right: WorldPoint::new(self.x + self.width, self.y + self.height),
            bottom_left: WorldPoint::new(self.x, self.y + self.height),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum FlagOwnership {
    Nobody,
    A,
    B,
    Both,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct FlagsOwnership {
    ownerships: Vec<(FlagName, FlagOwnership)>,
}

impl FlagsOwnership {
    pub fn empty() -> Self {
        Self { ownerships: vec![] }
    }

    pub fn from_control(map: &Map, a_control: &MapControl, b_control: &MapControl) -> Self {
        let mut ownerships = vec![];

        for flag in map.flags() {
            let mut is_a_control =
                map.one_of_spawn_zone_contains_flag(&a_control.spawn_zone_names(), &flag);
            let mut is_b_control =
                map.one_of_spawn_zone_contains_flag(&b_control.spawn_zone_names(), &flag);

            if a_control.spawn_zone_names().contains(&SpawnZoneName::All) && !is_b_control {
                is_a_control = true;
            }
            if b_control.spawn_zone_names().contains(&SpawnZoneName::All) && !is_a_control {
                is_b_control = true;
            }

            let flag_ownership = match (is_a_control, is_b_control) {
                (true, true) => FlagOwnership::Both,
                (true, false) => FlagOwnership::A,
                (false, true) => FlagOwnership::B,
                (false, false) => FlagOwnership::Nobody,
            };
            ownerships.push((flag.name().clone(), flag_ownership));
        }

        Self { ownerships }
    }
}

#[cfg(test)]
pub mod test {
    use oc_core::spawn::SpawnZoneName;
    use rstest::*;

    use crate::map::{decor::*, spawn::*, *};
    use std::path::PathBuf;

    use super::*;

    #[cfg(test)]
    fn map(spawn_zones: Vec<SpawnZone>, flags: Vec<Flag>) -> Map {
        Map::new(
            "TestMap".to_string(),
            PathBuf::from("."),
            PathBuf::from("."),
            PathBuf::from("."),
            vec![],
            spawn_zones,
            10,
            10,
            vec![],
            1,
            1,
            Decor::new(vec![], vec![]),
            flags,
        )
    }

    #[cfg(test)]
    #[fixture]
    fn spawn_zones1() -> Vec<SpawnZone> {
        vec![SpawnZone::new(
            SpawnZoneName::North,
            0.,
            0.,
            10.,
            10.,
            10.,
            10.,
        )]
    }

    #[cfg(test)]
    #[fixture]
    fn flag1() -> Flag {
        Flag::new(FlagName("FlagName".to_string()), 1., 1., 8., 8.)
    }

    #[rstest]
    #[case(MapControl::new(vec![SpawnZoneName::North]), MapControl::new(vec![]), FlagOwnership::A)]
    #[case(MapControl::new(vec![]), MapControl::new(vec![SpawnZoneName::North]), FlagOwnership::B)]
    #[case(MapControl::new(vec![SpawnZoneName::North]), MapControl::new(vec![SpawnZoneName::North]), FlagOwnership::Both)]
    #[case(MapControl::new(vec![SpawnZoneName::North]), MapControl::new(vec![]), FlagOwnership::A)]
    #[case(MapControl::new(vec![]), MapControl::new(vec![]), FlagOwnership::Nobody)]
    fn flag_owned_by_a(
        spawn_zones1: Vec<SpawnZone>,
        flag1: Flag,
        #[case] a_control: MapControl,
        #[case] b_control: MapControl,
        #[case] ownership: FlagOwnership,
    ) {
        // Given
        let flags = vec![flag1];
        let map = map(spawn_zones1, flags);

        // When
        let flags_ownership = FlagsOwnership::from_control(&map, &a_control, &b_control);

        // Then
        assert_eq!(
            flags_ownership,
            FlagsOwnership {
                ownerships: vec![(FlagName("FlagName".to_string()), ownership)]
            }
        )
    }
}
