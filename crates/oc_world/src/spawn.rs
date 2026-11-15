use oc_root::geo::WorldVec2;
use oc_root::side::Side;
#[cfg(feature = "bevy")]
use oc_root::{WcfgInto, WorldConfig};
use oc_utils::polygon::Polygon;

use crate::flag::{Flags, Ownership};

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
#[serde(transparent)]
pub struct SpawnZoneName(pub String);

impl SpawnZoneName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct SpawnZone {
    pub name: SpawnZoneName,
    pub start: WorldVec2,
    pub points: Vec<WorldVec2>,
}

impl Polygon for SpawnZone {
    fn points(&self) -> &[WorldVec2] {
        &self.points
    }
}

#[cfg(feature = "bevy")]
impl SpawnZone {
    pub fn bevy_polygon(&self, w: &WorldConfig) -> geo::Polygon<f32> {
        let points: Vec<geo::Coord<f32>> = self
            .points
            .iter()
            .map(|p| {
                use oc_root::geo::ScreenVec2;

                let p: ScreenVec2 = (*p).into_(w);
                geo::Coord { x: p.x, y: p.y }
            })
            .collect();
        geo::Polygon::new(geo::LineString::new(points), vec![])
    }
}

/// Names of `zones` owned by `side` according to flags. A zone is owned when each flag in it
/// (by its center) is owned by `side`, or owned by nobody while the zone was already owned.
/// A zone without flag keeps its previous ownership.
pub fn owned_spawn_zones(
    zones: &[SpawnZone],
    owned: &[SpawnZone],
    flags: &Flags,
    side: Side,
) -> Vec<SpawnZoneName> {
    zones
        .iter()
        .filter(|zone| {
            let already = owned.iter().any(|z| z.name == zone.name);
            let mut ownerships = flags
                .iter()
                .filter(|(flag, _)| zone.contains(flag.center()))
                .map(|(_, ownership)| ownership)
                .peekable();

            // Zone without flag keeps its ownership
            if ownerships.peek().is_none() {
                return already;
            }

            ownerships.all(|ownership| match (ownership, side) {
                (Ownership::A, Side::A) | (Ownership::B, Side::B) => true,
                (Ownership::Nobody, _) => already,
                (Ownership::A, Side::B) | (Ownership::B, Side::A) | (Ownership::Both, _) => false,
            })
        })
        .map(|zone| zone.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flag::{Flag, FlagName};

    fn zone(name: &str, x: f32) -> SpawnZone {
        SpawnZone {
            name: SpawnZoneName::new(name),
            start: WorldVec2::new(x, 0.),
            points: vec![
                WorldVec2::new(x, 0.),
                WorldVec2::new(x + 100., 0.),
                WorldVec2::new(x + 100., 100.),
                WorldVec2::new(x, 100.),
            ],
        }
    }

    fn flag(x: f32, ownership: Ownership) -> (Flag, Ownership) {
        let flag = Flag::new(FlagName("F".to_string()), WorldVec2::new(x, 40.), 20., 20.);
        (flag, ownership)
    }

    fn names(values: &[&str]) -> Vec<SpawnZoneName> {
        values.iter().map(|v| SpawnZoneName::new(*v)).collect()
    }

    #[test]
    fn owned_by_flags() {
        // Zones: Z1 [0;100], Z2 [200;300], Z3 [400;500], Z4 [600;700]
        let zones = vec![
            zone("Z1", 0.),
            zone("Z2", 200.),
            zone("Z3", 400.),
            zone("Z4", 600.),
        ];
        let a_owned = vec![zones[0].clone(), zones[1].clone()];
        let b_owned = vec![zones[2].clone(), zones[3].clone()];
        let flags = Flags::new(vec![
            // Z1: one A flag and one nobody flag: A keep it
            flag(10., Ownership::A),
            flag(60., Ownership::Nobody),
            // Z2: taken by B
            flag(240., Ownership::B),
            // Z3: one A flag and one B flag: nobody
            flag(410., Ownership::A),
            flag(460., Ownership::B),
            // Z4: disputed
            flag(640., Ownership::Both),
            // Flag center out of zones (rectangle overlaps Z1)
            flag(95., Ownership::B),
        ]);

        assert_eq!(
            owned_spawn_zones(&zones, &a_owned, &flags, Side::A),
            names(&["Z1"])
        );
        assert_eq!(
            owned_spawn_zones(&zones, &b_owned, &flags, Side::B),
            names(&["Z2"])
        );
    }

    #[test]
    fn nobody_flags() {
        let zones = vec![zone("Z1", 0.), zone("Z2", 200.)];
        let a_owned = vec![zones[0].clone()];
        let flags = Flags::new(vec![
            flag(10., Ownership::Nobody),
            flag(210., Ownership::Nobody),
        ]);

        // Nobody flags keep ownership, but don't give it
        assert_eq!(
            owned_spawn_zones(&zones, &a_owned, &flags, Side::A),
            names(&["Z1"])
        );
        assert_eq!(owned_spawn_zones(&zones, &[], &flags, Side::B), names(&[]));
    }

    #[test]
    fn without_flags() {
        let zones = vec![zone("Z1", 0.), zone("Z2", 200.)];
        let a_owned = vec![zones[0].clone()];
        let flags = Flags::default();

        // Zones without flag keep their ownership
        assert_eq!(
            owned_spawn_zones(&zones, &a_owned, &flags, Side::A),
            names(&["Z1"])
        );
        assert_eq!(owned_spawn_zones(&zones, &[], &flags, Side::B), names(&[]));
    }
}
