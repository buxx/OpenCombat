use glam::Vec2;
use oc_mod::{Mod, illustration::IllustrationIndex, nature::Traversability};
use oc_physics::{Force, IgnoreSide, Physic, collision::Material, volume::Volume};
use oc_root::{
    WorldConfig,
    geo::{WorldVec2, WorldVec3},
    material::MaterialKind,
    side::Side,
};
use oc_utils::d2::Direction;

pub mod network;

oc_utils::index_type! {
    pub struct VehicleIndex(pub u64);
}

// FIXME: manage multiple weapons (and so, reload per weapons)
/// Main weapon is loaded when its reload progress reach this value
pub const LOADED: f32 = 1.0;
/// Tolerance (pixels) to consider the vehicle reached its destination
pub const DESTINATION_TOLERANCE: f32 = 5.0;

#[oc_derive::archivable]
#[derive(Debug, PartialEq, Clone)]
pub struct Vehicle {
    pub side: Side,
    pub type_: oc_mod::vehicle::VehicleIndex,
    pub illustration: IllustrationIndex,
    pub position: WorldVec3,
    pub forces: Vec<Force>,
    pub chassis_direction: Direction,
    pub turret_direction: Option<Direction>,
    pub chassis_target: Option<Direction>,
    pub turret_target: Option<Direction>,
    pub destination: Option<WorldVec2>,
    // FIXME BS NOW: refactor U8Progress to be f32 Progress and use it.
    pub reload_progress: f32,
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum Update {
    SetPosition(WorldVec3),
    SetForces(Vec<Force>),
    SetChassisOrientation(Direction),
    SetTurretDirection(Direction),
    SetChassisTarget(Option<Direction>),
    SetTurretTarget(Option<Direction>),
    SetDestination(Option<WorldVec2>),
    SetReloadProgress(f32),
}

impl Vehicle {
    pub fn new(
        side: Side,
        type_: oc_mod::vehicle::VehicleIndex,
        illustration: IllustrationIndex,
        position: WorldVec3,
        forces: Vec<Force>,
        chassis_direction: Direction,
        turret_direction: Option<Direction>,
    ) -> Self {
        Self {
            side,
            type_,
            illustration,
            position,
            forces,
            chassis_direction,
            turret_direction,
            chassis_target: None,
            turret_target: None,
            destination: None,
            reload_progress: LOADED,
        }
    }

    pub fn with_reload_progress(mut self, value: f32) -> Self {
        self.reload_progress = value;
        self
    }

    pub fn loaded(&self) -> bool {
        self.reload_progress >= LOADED
    }

    pub fn apply(&mut self, update: &Update) {
        match update {
            Update::SetPosition(position) => self.position = *position,
            Update::SetForces(forces) => self.forces = forces.clone(),
            Update::SetChassisOrientation(direction) => self.chassis_direction = *direction,
            Update::SetTurretDirection(direction) => self.turret_direction = Some(*direction),
            Update::SetChassisTarget(direction) => self.chassis_target = *direction,
            Update::SetTurretTarget(direction) => self.turret_target = *direction,
            Update::SetDestination(destination) => self.destination = *destination,
            Update::SetReloadProgress(progress) => self.reload_progress = *progress,
        }
    }

    /// Chassis and turret directions after rotating `delta` seconds toward their targets (at `type_` rotation speeds).
    pub fn rotate(
        &self,
        type_: &oc_mod::vehicle::Vehicle,
        delta: f32,
    ) -> (Direction, Option<Direction>) {
        let chassis = match self.chassis_target {
            Some(target) => self
                .chassis_direction
                .rotated_toward(&target, type_.chassis_rotation_speed.radians(delta)),
            None => self.chassis_direction,
        };
        let turret = match (self.turret_direction, self.turret_target) {
            (Some(turret), Some(target)) => {
                Some(turret.rotated_toward(&target, type_.turret_rotation_speed.radians(delta)))
            }
            (turret, _) => turret,
        };

        (chassis, turret)
    }

    /// Position after translating `delta` seconds with forces, stopping at destination. Ignore
    /// collisions: used by clients to animate vehicles between server updates.
    pub fn translate(&self, w: &WorldConfig, delta: f32) -> WorldVec3 {
        let mut position = self.position;

        for force in &self.forces {
            let Force::Translation(direction, speed) = force;
            let pixels = speed.0 * delta * w.geo_pixels_per_meters();
            let mut next = WorldVec3::new(
                position.x + direction.x * pixels,
                position.y + direction.y * pixels,
                position.z,
            );

            if let Some(destination) = self.destination {
                let destination = Vec2::from(destination);
                let remaining = Vec2::from(WorldVec2::from(position)).distance(destination);
                if remaining <= pixels {
                    next = WorldVec3::new(destination.x, destination.y, position.z);
                }
            }

            position = next;
        }

        position
    }
}

impl Physic for Vehicle {
    fn position(&self, _: &WorldConfig) -> WorldVec3 {
        self.position
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        &self.forces
    }

    fn volumes(
        &self,
        ref_: WorldVec3,
        _: bool,
        w: &WorldConfig,
        mod_: &Mod,
    ) -> Vec<(Volume, Traversability, Direction)> {
        // Chassis box, oriented with the chassis (width on x and length on y when facing north)
        let type_ = mod_.vehicle(self.type_);
        let (width, length) = type_.size;
        let cube = Volume::Cube {
            x: 0.,
            y: 0.,
            z: 0.,
            width: width.pixels(w),
            height: length.pixels(w),
            depth: type_.height.pixels(w),
        }
        .with_ref(ref_);

        vec![(
            cube,
            Traversability {
                individual: true,
                vehicle: false,
                projectile: false,
            },
            self.chassis_direction,
        )]
    }

    fn ignore_side(&self) -> IgnoreSide {
        IgnoreSide::None
    }

    fn side(&self) -> Option<Side> {
        Some(self.side)
    }

    fn ignore_underground(&self) -> bool {
        true
    }

    fn receive_z(&self) -> bool {
        true
    }

    fn want_stop(&self) -> Option<(WorldVec2, f32)> {
        self.destination.map(|d| (d, DESTINATION_TOLERANCE))
    }
}

impl<'a> oc_utils::collections::WithIds<VehicleIndex, &'a Vehicle> for &'a [Vehicle] {
    fn with_ids(&self) -> Vec<(VehicleIndex, &'a Vehicle)> {
        self.iter()
            .enumerate()
            .map(|(i, vehicle)| (VehicleIndex(i as u64), vehicle))
            .collect()
    }
}

impl Material for Vehicle {
    fn kind(&self) -> Option<MaterialKind> {
        Some(MaterialKind::Vehicle)
    }
}

#[cfg(test)]
mod tests {
    use oc_root::physics::{DegreesPerSeconds, Meters, MetersSeconds, Seconds};

    use super::*;

    fn type_() -> oc_mod::vehicle::Vehicle {
        oc_mod::vehicle::Vehicle {
            name: "Tank".to_string(),
            row: 0,
            crew: vec![],
            size: (Meters(2.), Meters(4.)),
            height: Meters(2.),
            speed: MetersSeconds(2.),
            chassis_rotation_speed: DegreesPerSeconds(90.),
            turret_rotation_speed: DegreesPerSeconds(45.),
            reload_duration: Seconds(4.),
        }
    }

    fn vehicle() -> Vehicle {
        Vehicle::new(
            Side::A,
            oc_mod::vehicle::VehicleIndex(0),
            IllustrationIndex(0),
            WorldVec3::new(10., 10., 0.),
            vec![],
            Direction::NORTH,
            Some(Direction::NORTH),
        )
    }

    #[test]
    fn rotate_toward_targets() {
        // Given: chassis and turret must rotate to est (90°)
        let mut vehicle = vehicle();
        vehicle.chassis_target = Some(Direction::EST);
        vehicle.turret_target = Some(Direction::EST);

        // When: half a second
        let (chassis, turret) = vehicle.rotate(&type_(), 0.5);

        // Then: chassis rotated 45°, turret 22.5°
        assert!((Direction::NORTH.angle_to(&chassis).abs() - 45f32.to_radians()).abs() < 1e-4);
        let turret = turret.unwrap();
        assert!((Direction::NORTH.angle_to(&turret).abs() - 22.5f32.to_radians()).abs() < 1e-4);

        // When: enough time, targets are reached
        let (chassis, turret) = vehicle.rotate(&type_(), 2.);
        assert_eq!(chassis, Direction::EST);
        assert_eq!(turret, Some(Direction::EST));
    }

    #[test]
    fn rotate_without_turret() {
        let mut vehicle = vehicle();
        vehicle.turret_direction = None;
        vehicle.turret_target = Some(Direction::EST);

        let (chassis, turret) = vehicle.rotate(&type_(), 1.);

        assert_eq!(chassis, Direction::NORTH);
        assert_eq!(turret, None);
    }

    #[test]
    fn translate_stops_at_destination() {
        // Given: 1 pixel per meter, moving north at 2m/s
        let w = WorldConfig::new(100, 100).with_geo_pixels_per_meters(1.);
        let mut vehicle = vehicle();
        vehicle.forces = vec![Force::Translation(
            WorldVec3::new(0., 1., 0.),
            MetersSeconds(2.),
        )];

        // When / Then: free translation
        assert_eq!(vehicle.translate(&w, 1.), WorldVec3::new(10., 12., 0.));

        // When / Then: stops on destination
        vehicle.destination = Some(WorldVec2::new(10., 11.));
        assert_eq!(vehicle.translate(&w, 1.), WorldVec3::new(10., 11., 0.));
    }
}
