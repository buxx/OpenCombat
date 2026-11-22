use glam::Vec2;
use oc_mod::{Mod, illustration::IllustrationIndex, nature::Traversability, shell::ShellIndex};
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
/// Main weapon is aimed when its aim progress reach this value
pub const AIMED: f32 = 1.0;
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
    /// Gunner aiming progress (turret oriented to the target), fire when reach [`AIMED`]
    pub aim_progress: f32,
    /// Shells stored in the vehicle: count by shell type
    pub shells: Vec<(ShellIndex, u32)>,
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
    SetAimProgress(f32),
    SetShells(Vec<(ShellIndex, u32)>),
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
            aim_progress: 0.,
            shells: vec![],
        }
    }

    pub fn with_reload_progress(mut self, value: f32) -> Self {
        self.reload_progress = value;
        self
    }

    pub fn with_shells(mut self, value: Vec<(ShellIndex, u32)>) -> Self {
        self.shells = value;
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
            Update::SetAimProgress(progress) => self.aim_progress = *progress,
            Update::SetShells(shells) => self.shells = shells.clone(),
        }
    }

    /// Chassis and turret directions after rotating `delta` seconds toward their targets (at `type_` rotation speeds).
    /// Turret is mounted on the chassis: it is carried by the chassis rotation, then rotates (relatively to
    /// the chassis) toward its target.
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
        let turret = self.turret_direction.map(|turret| {
            let carried = match chassis == self.chassis_direction {
                true => turret,
                false => {
                    let angle = self.chassis_direction.angle_to(&chassis);
                    let rotation = Direction::new(angle.cos(), angle.sin());
                    let (x, y) = rotation.rotate(turret.x, turret.y);
                    Direction::new(x, y)
                }
            };
            match self.turret_target {
                Some(target) => {
                    carried.rotated_toward(&target, type_.turret_rotation_speed.radians(delta))
                }
                None => carried,
            }
        });

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

    // WARNING: AT generated
    /// Whether `point` is on the chassis footprint (considering chassis orientation),
    /// enlarged by `tolerance` pixels on each side
    pub fn chassis_contains(
        &self,
        type_: &oc_mod::vehicle::Vehicle,
        w: &WorldConfig,
        point: WorldVec2,
        tolerance: f32,
    ) -> bool {
        let (width, length) = type_.size;
        let (half_width, half_length) = (width.pixels(w) / 2., length.pixels(w) / 2.);
        let forward = Vec2::new(self.chassis_direction.x, self.chassis_direction.y)
            .try_normalize()
            .unwrap_or(Vec2::new(Direction::NORTH.x, Direction::NORTH.y));
        let lateral = forward.perp();
        let to = Vec2::from(point) - Vec2::new(self.position.x, self.position.y);

        to.dot(forward).abs() <= half_length + tolerance
            && to.dot(lateral).abs() <= half_width + tolerance
    }

    // WARNING: AI generated
    /// Chassis points an opposite individual can see: chassis center and corners (considering
    /// chassis orientation). Seeing one of them is seeing the vehicle.
    pub fn chassis_sight_points(&self, w: &WorldConfig, mod_: &Mod) -> Vec<WorldVec3> {
        let type_ = mod_.vehicle(self.type_);
        let (width, length) = type_.size;
        let (half_width, half_length) = (width.pixels(w) / 2., length.pixels(w) / 2.);
        let z = self.position.z + type_.height.pixels(w) / 2.;
        let forward = Vec2::new(self.chassis_direction.x, self.chassis_direction.y)
            .try_normalize()
            .unwrap_or(Vec2::new(Direction::NORTH.x, Direction::NORTH.y));
        let lateral = forward.perp();
        let center = Vec2::new(self.position.x, self.position.y);

        [(0., 0.), (1., 1.), (1., -1.), (-1., 1.), (-1., -1.)]
            .into_iter()
            .map(|(f, l)| {
                let point = center + forward * (f * half_length) + lateral * (l * half_width);
                WorldVec3::new(point.x, point.y, z)
            })
            .collect()
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
            aim_duration: Seconds(1.),
            penetration_resistance: 1.,
            compatible_shell: vec![],
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
    fn chassis_contains_oriented() {
        // Given: chassis (2m x 4m) facing est, so 4m along x and 2m along y
        let w = WorldConfig::new(100, 100);
        let type_ = type_();
        let mut vehicle = vehicle();
        vehicle.chassis_direction = Direction::EST;
        let (x, y) = (vehicle.position.x, vehicle.position.y);
        let ppm = w.geo_pixels_per_meters();

        // Then
        assert!(vehicle.chassis_contains(&type_, &w, WorldVec2::new(x, y), 0.));
        assert!(vehicle.chassis_contains(&type_, &w, WorldVec2::new(x + 1.9 * ppm, y), 0.));
        assert!(!vehicle.chassis_contains(&type_, &w, WorldVec2::new(x, y + 1.9 * ppm), 0.));
        assert!(vehicle.chassis_contains(&type_, &w, WorldVec2::new(x, y + 1.9 * ppm), ppm));
    }

    #[test]
    fn rotate_toward_targets() {
        // Given: chassis and turret must rotate to est (90°)
        let mut vehicle = vehicle();
        vehicle.chassis_target = Some(Direction::EST);
        vehicle.turret_target = Some(Direction::EST);

        // When: a quarter of second
        let (chassis, turret) = vehicle.rotate(&type_(), 0.25);

        // Then: chassis rotated 22.5°, turret carried by chassis (22.5°) plus its own 11.25°
        assert!((Direction::NORTH.angle_to(&chassis).abs() - 22.5f32.to_radians()).abs() < 1e-4);
        let turret = turret.unwrap();
        assert!((Direction::NORTH.angle_to(&turret).abs() - 33.75f32.to_radians()).abs() < 1e-4);

        // When: enough time, targets are reached
        let (chassis, turret) = vehicle.rotate(&type_(), 2.);
        assert_eq!(chassis, Direction::EST);
        assert_eq!(turret, Some(Direction::EST));
    }

    #[test]
    fn turret_follows_chassis_rotation() {
        // Given: turret without target, chassis must rotate to est (90°)
        let mut vehicle = vehicle();
        vehicle.turret_direction = Some(Direction::WEST);
        vehicle.chassis_target = Some(Direction::EST);

        // When: half a second
        let (chassis, turret) = vehicle.rotate(&type_(), 0.5);

        // Then: turret rotated as much as the chassis (45° clockwise)
        let turret = turret.unwrap();
        assert!((Direction::NORTH.angle_to(&chassis) - 45f32.to_radians()).abs() < 1e-4);
        assert!((Direction::WEST.angle_to(&turret) - 45f32.to_radians()).abs() < 1e-4);
    }

    #[test]
    fn turret_compensates_chassis_rotation_to_keep_target() {
        // Given: turret on its target (north) while chassis rotates to est
        let mut vehicle = vehicle();
        vehicle.chassis_target = Some(Direction::EST);
        vehicle.turret_target = Some(Direction::NORTH);
        let type_ = oc_mod::vehicle::Vehicle {
            chassis_rotation_speed: DegreesPerSeconds(30.),
            ..type_()
        };

        // When
        let (_, turret) = vehicle.rotate(&type_, 0.5);

        // Then: turret faster than chassis, it stays on target
        assert_eq!(turret, Some(Direction::NORTH));
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
