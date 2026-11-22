use crate::{collision::Material, volume::Volume};
use line_drawing::Bresenham3d;
use oc_mod::{Mod, nature::Traversability};
use oc_root::{
    WcfgFrom, WorldConfig,
    geo::{WorldVec2, WorldVec3},
    material::MaterialKind,
    physics::MetersSeconds,
    side::Side,
    y::V,
};
use oc_utils::{
    d2::{Direction, Xy},
    number::almost_equal,
};
use std::fmt::Debug;

pub mod collision;
pub mod corps;
pub mod effect;
pub mod fx;
pub mod reactive;
pub mod translation;
pub mod update;
pub mod volume;

/// Stop point is considered on the way of a translation when it is at most this factor of its
/// tolerance away from the translation line
const WANT_STOP_LINE_TOLERANCE: f32 = 2.0;

type PhysicVolumes = Vec<(Volume, Traversability, Direction)>;

pub trait Physic: Material {
    // TODO: maby position should be `Geo` instead `Physics`...
    fn position(&self, w: &WorldConfig) -> WorldVec3;
    fn forces(&self, w: &WorldConfig) -> &Vec<Force>;
    fn volumes(
        &self,
        ref_: WorldVec3,
        include_underground: bool,
        w: &WorldConfig,
        mod_: &Mod,
    ) -> PhysicVolumes;
    fn ignore_side(&self) -> IgnoreSide;
    fn side(&self) -> Option<Side> {
        None
    }
    fn intercept(&self, _kind: MaterialKind, _mod_: &Mod) -> f32 {
        0.0
    }
    fn collision_ignorable(&self) -> bool {
        false
    }
    fn collision_ignore_origin(&self) -> Option<WorldVec3> {
        None
    }
    fn emit_proximity(&self) -> bool {
        false
    }
    fn receive_proximity(&self) -> bool {
        false
    }
    fn ignore_underground(&self) -> bool {
        false
    }
    fn apply_z(&self, _: &WorldConfig) -> Option<f32> {
        None
    }
    fn receive_z(&self) -> bool {
        false
    }
    fn want_stop(&self) -> Option<(WorldVec2, f32)> {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IgnoreSide {
    None,
    All,
    Side(Side),
}

impl IgnoreSide {
    fn ignore(&self, side: Side) -> bool {
        match self {
            IgnoreSide::None => false,
            IgnoreSide::All => true,
            IgnoreSide::Side(side_) => *side_ == side,
        }
    }
}

pub trait UpdatePhysic: Physic + Material {
    fn set_position(&mut self, value: WorldVec3);
    fn push_force(&mut self, value: Force);
    fn remove_force(&mut self, value: &Force);
    fn set_volumes(&self, value: Vec<(Volume, Traversability)>);
}

// TODO: gravité
#[oc_derive::archivable]
#[derive(Debug, PartialEq, Clone)]
pub enum Force {
    Translation(WorldVec3, MetersSeconds),
}

/// Ratio of a physics tick elapsed when an event occurred.
#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct TickProgress(pub f32);

impl PartialEq for TickProgress {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Event<T: serde::Serialize> {
    /// Physics application result position outside the world
    NoTile(T),
    /// Physics application result a collide between two objects
    /// With progress of the tick when the collision occurred
    Collision(T, T, TickProgress),
    /// Object which emit proximity is on same tile than other
    Proximity(ProximityKind, T, T, [f32; 3]),
    /// Object want stop here by its own decision
    WantStop(T, WorldVec3),
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum ProximityKind {
    Fly,
    Impact,
}

pub trait World<Z> {
    fn at(&self, xy: Xy) -> Vec<(Z, Box<dyn Physic>)>;
}

/// Apply forces of `object` during `delta` seconds.
///
/// Returns the new position, the forces to keep for next step and the emitted events.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn step<'a, I, O, C, P, Z>(
    w: &WorldConfig,
    delta: f32,
    mod_: &Mod,
    object: (I, &'a O),
    collision_at: C,
    proximity_at: P,
    collision_ignore_pixels: usize,
    origin: &str,
) -> (WorldVec3, Vec<Force>, Vec<Event<Z>>)
where
    I: Clone + Into<Z> + Debug,
    O: Physic,
    C: Fn(Xy) -> Vec<(Z, &'a dyn Physic)>,
    P: Fn(Xy) -> Vec<(Z, &'a dyn Physic)>,
    Z: Debug + serde::Serialize + PartialEq + Copy,
{
    let (i, object) = object;
    Stepper {
        w,
        delta,
        mod_,
        origin,
        id: i.clone().into(),
        i,
        object,
        kind: object.kind(),
        emit_proximity: object.emit_proximity(),
        ignore_underground: object.ignore_underground(),
        receive_z: object.receive_z(),
        want_stop: object.want_stop(),
        collision_at,
        proximity_at,
        collision_ignore_pixels,
        collision_ignore_origin: object.collision_ignore_origin(),
        apply_z: None,
        events: vec![],
        progress_from: WorldVec3::new(0., 0., 0.),
        progress_total: 0.0,
        tile_cache: None,
    }
    .run()
}

/// Ratio of a translation of `total` pixels done when reaching `to` from `from`
fn progress(from: WorldVec3, to: WorldVec3, total: f32) -> f32 {
    if total <= 0.0 {
        return 0.0;
    }
    (from.distance(to) / total).clamp(0.0, 1.0)
}

/// What happened to a force once its translation has been simulated.
enum Translation {
    /// Force is still applicable on next step (end reached, or world border hit)
    Kept(WorldVec3),
    /// Force is consumed (collision or want stop): object stops at given position
    Dropped(WorldVec3),
}

/// What happen to a new pixel compute when compute line of translation
enum PixelOutcome {
    Pass,
    Stop,
}

struct Stepper<'a, 's, I, O, C, P, Z: serde::Serialize> {
    // Inputs
    w: &'s WorldConfig,
    delta: f32,
    mod_: &'s Mod,
    origin: &'s str,
    i: I,
    id: Z,
    object: &'a O,
    collision_at: C,
    proximity_at: P,
    collision_ignore_pixels: usize,
    collision_ignore_origin: Option<WorldVec3>,

    // Object properties
    kind: Option<MaterialKind>,
    emit_proximity: bool,
    ignore_underground: bool,
    receive_z: bool,
    want_stop: Option<(WorldVec2, f32)>,

    // Accumulated state
    apply_z: Option<f32>,
    events: Vec<Event<Z>>,
    /// Start and total length (pixels) of the current translation, to compute progress on events
    progress_from: WorldVec3,
    progress_total: f32,
    /// Others objects of the last tile tested (pixels of a tile share them)
    tile_cache: Option<TileCache<'a, Z>>,
}

/// What `collision_at` and `proximity_at` returned for a tile, with lazily computed volumes
struct TileCache<'a, Z> {
    xy: Xy,
    collisions: Vec<(Z, &'a dyn Physic)>,
    proximities: Vec<(Z, &'a dyn Physic)>,
    volumes: Vec<Option<PhysicVolumes>>,
    /// For each collision object: interception already rolled on this tile (see [`Stepper::intercepted_by`])
    intercept: Vec<bool>,
}

impl<'a, I, O, C, P, Z> Stepper<'a, '_, I, O, C, P, Z>
where
    I: Clone + Into<Z> + Debug,
    O: Physic,
    C: Fn(Xy) -> Vec<(Z, &'a dyn Physic)>,
    P: Fn(Xy) -> Vec<(Z, &'a dyn Physic)>,
    Z: Debug + serde::Serialize + PartialEq + Copy,
{
    fn run(mut self) -> (WorldVec3, Vec<Force>, Vec<Event<Z>>) {
        let w = self.w;
        let mut position = self.object.position(w);
        let mut forces = vec![];
        tracing::trace!(name="physics-step-start", origin=self.origin, i=?self.i, p=?position, forces=?self.object.forces(w));

        for force in self.object.forces(w) {
            match force {
                Force::Translation(direction, speed) => {
                    match self.translate(position, direction, speed) {
                        Translation::Kept(new_position) => {
                            position = new_position;
                            forces.push(force.clone());
                        }
                        Translation::Dropped(new_position) => position = new_position,
                    }
                }
            }
        }

        tracing::trace!(name="physics-step-finished", position=?position, forces=?forces, events=self.events.len());
        (position, forces, self.events)
    }

    /// Simulate a translation force pixel by pixel from `position`.
    fn translate(
        &mut self,
        position: WorldVec3,
        direction: &WorldVec3,
        speed: &MetersSeconds,
    ) -> Translation {
        let (origin, i) = (self.origin, self.i.clone());
        let mut pixels = speed.0 * self.delta * self.w.geo_pixels_per_meters();
        let [x, y, z] = position.into();

        // Stop point on the way: translation ends exactly on it.
        let stop = self.want_stop_on_way(position, direction, pixels);
        if let Some(distance) = stop {
            pixels = distance;
        }

        let (x_, y_, z_) = match (stop, self.want_stop) {
            (Some(_), Some((p, _))) => (p.x, p.y, z + direction.z * pixels),
            _ => (
                x + direction.x * pixels,
                y + direction.y * pixels,
                z + direction.z * pixels,
            ),
        };

        tracing::trace!(
            name = "physics-step-translation-start", origin=origin, i=?i,
            x = x, y = y, z = z, x_ = x_, y_ = y_, z_ = z_,
            pixels = pixels,
            ignore_underground = self.ignore_underground,
        );

        let start = (x.ceil() as isize, y.ceil() as isize, z.ceil() as isize);
        let end = (x_.ceil() as isize, y_.ceil() as isize, z_.ceil() as isize);
        let world_width = self.w.world_width_pixels() as isize;
        let world_height = self.w.world_height_pixels() as isize;

        self.progress_from = WorldVec3::new(x, y, z);
        self.progress_total = pixels;

        let mut position = position;
        for (pixel_x, pixel_y, pixel_z) in Bresenham3d::new(start, end) {
            if pixel_x < 0 || pixel_y < 0 || pixel_x >= world_width || pixel_y >= world_height {
                tracing::trace!(name="physics-step-translation-outside", origin=origin, i=?i, pixel=?(pixel_x, pixel_y, pixel_z));
                self.events.push(Event::NoTile(self.id));
                return Translation::Dropped(position);
            }

            let pixel: WorldVec3 = [pixel_x as f32, pixel_y as f32, pixel_z as f32].into();
            let xy = Xy::from_((pixel_x, pixel_y), self.w);
            position = pixel;

            tracing::trace!(name="physics-step-translation-line-pixel", origin=origin, i=?i, pixel=?pixel, xy=?xy);

            if let PixelOutcome::Stop = self.step_pixel(pixel, xy) {
                return Translation::Dropped(pixel);
            }
        }

        // Not interrupted: position is now end of translation (bresenham3d accept only isize)
        let mut position: WorldVec3 = [x_, y_, z_].into();
        if let Some(z) = self.apply_z {
            position.z = z;
        }

        if stop.is_some() {
            tracing::trace!(name="physics-step-translation-want-stop-reached", origin=origin, i=?i, p=?position);
            self.events.push(Event::WantStop(self.id, position));
            return Translation::Dropped(position);
        }

        tracing::trace!(name="physics-step-translation-updated", origin=origin, i=?i, p=?position);
        Translation::Kept(position)
    }

    // WARNING: AI generated (fix bug of soldier walking around its move step)
    /// Distance (pixels) to the stop point when it is reached by a translation of `pixels` from
    /// `position` in `direction` (stop point near the translation line, at most
    /// [`WANT_STOP_LINE_TOLERANCE`] times its tolerance away)
    fn want_stop_on_way(
        &self,
        position: WorldVec3,
        direction: &WorldVec3,
        pixels: f32,
    ) -> Option<f32> {
        let (stop, tolerance) = self.want_stop?;
        let direction = glam::Vec2::new(direction.x, direction.y);
        let length = direction.length();
        if length <= f32::EPSILON {
            return None;
        }
        let direction = direction / length;
        let to = glam::Vec2::new(stop.x - position.x, stop.y - position.y);
        let along = to.dot(direction);
        let aside = (to - direction * along).length();
        // Translation pixels are along the full (3d) direction
        let distance = along / length;
        (along >= 0.0 && distance <= pixels && aside <= tolerance * WANT_STOP_LINE_TOLERANCE)
            .then_some(distance)
    }

    /// Test object at `pixel` (on tile `xy`): want stop, proximities and collisions.
    fn step_pixel(&mut self, pixel: WorldVec3, xy: Xy) -> PixelOutcome {
        if self.reached_want_stop(pixel) {
            tracing::trace!(name="physics-step-translation-line-pixel-want-stop-stop", origin=self.origin, i=?self.i, pixel=?pixel, xy=?xy);
            self.events.push(Event::WantStop(self.id, pixel));
            return PixelOutcome::Stop;
        }

        let volumes = self
            .object
            .volumes(pixel, !self.ignore_underground, self.w, self.mod_);

        let mut cache = match self.tile_cache.take() {
            Some(cache) if cache.xy == xy => cache,
            _ => {
                let collisions = (self.collision_at)(xy);
                TileCache {
                    xy,
                    volumes: collisions.iter().map(|_| None).collect(),
                    intercept: collisions.iter().map(|_| false).collect(),
                    collisions,
                    proximities: (self.proximity_at)(xy),
                }
            }
        };

        self.push_proximity_events(&cache.proximities, ProximityKind::Fly, pixel);

        let mut outcome = PixelOutcome::Pass;
        for (k, (o, other)) in cache.collisions.iter().enumerate() {
            if self.collide_with(
                *o,
                *other,
                pixel,
                &volumes,
                &mut cache.volumes[k],
                &mut cache.intercept[k],
            ) {
                let progress = progress(self.progress_from, pixel, self.progress_total);
                self.events
                    .push(Event::Collision(self.id, *o, TickProgress(progress)));
                self.push_proximity_events(&cache.proximities, ProximityKind::Impact, pixel);
                outcome = PixelOutcome::Stop;
                break;
            }
        }

        self.tile_cache = Some(cache);
        outcome
    }

    fn reached_want_stop(&self, pixel: WorldVec3) -> bool {
        self.want_stop.is_some_and(|(p, tolerance)| {
            almost_equal(p.x, pixel.x, tolerance) && almost_equal(p.y, pixel.y, tolerance)
        })
    }

    /// Does object (whose volumes at `pixel` are `volumes`) collide with `other` (identified by `o`) ?
    fn collide_with(
        &mut self,
        o: Z,
        other: &dyn Physic,
        pixel: WorldVec3,
        volumes: &PhysicVolumes,
        other_volumes: &mut Option<PhysicVolumes>,
        intercept_rolled: &mut bool,
    ) -> bool {
        let (origin, i) = (self.origin, &self.i);
        tracing::trace!(name="physics-step-translation-collision-other", origin=origin, i=?i, o=?o);

        // Do not test collision or else with itself (it is possible than `collision_at` return it)
        // NOTE: Maybe not the most optimized thing ?
        if o == self.id {
            return false;
        }

        if self.receive_z
            && let Some(z) = other.apply_z(self.w)
        {
            self.apply_z = Some(z);
        }

        if self.ignores_side_of(other) {
            tracing::trace!(name="physics-step-translation-collision-ignore-side", origin=origin, i=?i, o=?o);
            return false;
        }

        // This permit to ignore tile collisions near the object origin (z edge problem)
        if other.collision_ignorable() && self.in_collision_ignore_range(pixel) {
            return false;
        }

        let volumes2 = other_volumes.get_or_insert_with(|| {
            other.volumes(
                other.position(self.w),
                !self.ignore_underground,
                self.w,
                self.mod_,
            )
        });

        self.intercepted_by(other, volumes2, pixel, intercept_rolled)
            || self.volumes_collide(volumes, volumes2, o)
    }

    /// Is `pixel` close enough to the object origin to ignore collisions
    fn in_collision_ignore_range(&self, pixel: WorldVec3) -> bool {
        self.collision_ignore_origin.is_some_and(|origin| {
            let (dx, dy) = (pixel.x - origin.x, pixel.y - origin.y);
            (dx * dx + dy * dy).sqrt() < self.collision_ignore_pixels as f32
        })
    }

    /// Random interception, computed only if pixel z is within one of other volumes,
    /// whatever their traversability.
    fn intercepted_by(
        &self,
        other: &dyn Physic,
        volumes2: &PhysicVolumes,
        pixel: WorldVec3,
        rolled: &mut bool,
    ) -> bool {
        if *rolled {
            return false;
        }

        let intercept = self
            .kind
            .map_or(0.0, |kind| other.intercept(kind, self.mod_));

        if intercept == 0.0
            || !volumes2
                .iter()
                .any(|(volume, _, _)| volume.contains_z(pixel.z))
        {
            return false;
        }

        *rolled = true;

        if fastrand::f32() < intercept {
            tracing::trace!(name="physics-step-translation-intercept", origin=self.origin, i=?self.i, p=?pixel);
            return true;
        }

        false
    }

    fn volumes_collide(&self, volumes: &PhysicVolumes, volumes2: &PhysicVolumes, o: Z) -> bool {
        let (origin, i) = (self.origin, &self.i);

        volumes.iter().any(|(volume1, _, direction1)| {
            volumes2.iter().any(|(volume2, traversability2, direction2)| {
                // Test volumes collision only if object own a kind and other own too, and prohibe it on its tile
                if self.kind.is_none_or(|kind| traversability2.allow(kind)) {
                    tracing::trace!(name="physics-step-translation-collision-prohibe-allow", origin=origin, i=?i);
                    return false;
                }

                tracing::trace!(name="physics-step-translation-collision-test-collide-with", origin=origin, i=?i, o=?o, volume1=?volume1, volume2=?volume2);
                let collide = volume1.collide(
                    direction1.quat(V::Server),
                    volume2,
                    direction2.quat(V::Server),
                );
                if collide {
                    tracing::trace!(name="physics-step-translation-collision-collide", origin=origin, i=?i, o=?o);
                }
                collide
            })
        })
    }

    /// Must `other` be ignored because of its side ?
    fn ignores_side_of(&self, other: &dyn Physic) -> bool {
        other
            .side()
            .is_some_and(|side| self.object.ignore_side().ignore(side))
    }

    /// Emit proximity events for each of `others` if object emit and other receive.
    fn push_proximity_events(
        &mut self,
        others: &[(Z, &dyn Physic)],
        kind: ProximityKind,
        pixel: WorldVec3,
    ) {
        for (o, other) in others {
            tracing::trace!(name="physics-step-translation-proximity-other", origin=self.origin, i=?self.i, o=?o);

            // Do not test proximity or else with itself (it is possible than `proximity_at` return it)
            // NOTE: Maybe not the most optimized thing ?
            if *o == self.id {
                continue;
            }

            if self.ignores_side_of(*other) {
                tracing::trace!(name="physics-step-translation-ignore-side", origin=self.origin, i=?self.i, o=?o);
                continue;
            }

            if self.emit_proximity && other.receive_proximity() {
                self.events
                    .push(Event::Proximity(kind, self.id, *o, pixel.into()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use oc_geo::tile::TileXy;
    use oc_root::{WcfgInto, material::MaterialKind, utils::Frequency};

    use super::*;

    fn workspace_root() -> PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    struct MyObject(WorldVec3, Vec<Force>);
    #[derive(Debug, Clone, Copy, serde::Serialize, PartialEq)]
    struct MyObjectId(usize);

    impl Physic for MyObject {
        fn position(&self, _: &WorldConfig) -> WorldVec3 {
            self.0
        }
        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            &self.1
        }

        fn volumes(
            &self,
            ref_: WorldVec3,
            _: bool,
            _: &WorldConfig,
            _: &Mod,
        ) -> Vec<(Volume, Traversability, Direction)> {
            vec![(
                Volume::Point {
                    x: ref_.x,
                    y: ref_.y,
                    z: ref_.z,
                },
                Traversability::all(),
                Direction::IDENTITY,
            )]
        }

        fn ignore_side(&self) -> IgnoreSide {
            IgnoreSide::None
        }

        fn side(&self) -> Option<Side> {
            None
        }

        fn collision_ignore_origin(&self) -> Option<WorldVec3> {
            Some(self.0)
        }
    }

    impl Material for MyObject {
        fn kind(&self) -> Option<MaterialKind> {
            Some(MaterialKind::Projectile)
        }
    }

    struct MyTile(TileXy, Traversability);

    impl Physic for MyTile {
        fn position(&self, w: &WorldConfig) -> WorldVec3 {
            self.0.into_(w)
        }

        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            static EMPTY: Vec<Force> = vec![];
            &EMPTY
        }

        fn volumes(
            &self,
            ref_: WorldVec3,
            _: bool,
            w: &WorldConfig,
            _: &Mod,
        ) -> Vec<(Volume, Traversability, Direction)> {
            vec![(
                Volume::Cube {
                    x: ref_.x,
                    y: ref_.y,
                    z: ref_.z,
                    width: w.geo_pixels_per_tile() as f32,
                    height: w.geo_pixels_per_tile() as f32,
                    depth: f32::MAX,
                },
                self.1.clone(),
                Direction::IDENTITY,
            )]
        }

        fn ignore_side(&self) -> IgnoreSide {
            IgnoreSide::None
        }

        fn side(&self) -> Option<Side> {
            None
        }
    }

    impl Material for MyTile {
        fn kind(&self) -> Option<MaterialKind> {
            None
        }
    }

    struct MyIndividual(WorldVec3);

    impl Physic for MyIndividual {
        fn position(&self, _w: &WorldConfig) -> WorldVec3 {
            self.0
        }

        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            static EMPTY: Vec<Force> = vec![];
            &EMPTY
        }

        fn volumes(
            &self,
            ref_: WorldVec3,
            _: bool,
            _w: &WorldConfig,
            _: &Mod,
        ) -> Vec<(Volume, Traversability, Direction)> {
            vec![(
                Volume::Cube {
                    x: ref_.x,
                    y: ref_.y,
                    z: ref_.z,
                    width: 2.0,
                    height: 2.0,
                    depth: 10.,
                },
                Traversability::none(),
                Direction::IDENTITY,
            )]
        }

        fn ignore_side(&self) -> IgnoreSide {
            IgnoreSide::None
        }

        fn side(&self) -> Option<Side> {
            None
        }
    }

    impl Material for MyIndividual {
        fn kind(&self) -> Option<MaterialKind> {
            None
        }
    }

    struct MyIgnorableTile(TileXy, Traversability);

    impl Physic for MyIgnorableTile {
        fn position(&self, w: &WorldConfig) -> WorldVec3 {
            self.0.into_(w)
        }

        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            static EMPTY: Vec<Force> = vec![];
            &EMPTY
        }

        fn volumes(
            &self,
            ref_: WorldVec3,
            _: bool,
            w: &WorldConfig,
            _: &Mod,
        ) -> Vec<(Volume, Traversability, Direction)> {
            vec![(
                Volume::Cube {
                    x: ref_.x,
                    y: ref_.y,
                    z: ref_.z,
                    width: w.geo_pixels_per_tile() as f32,
                    height: w.geo_pixels_per_tile() as f32,
                    depth: f32::MAX,
                },
                self.1.clone(),
                Direction::IDENTITY,
            )]
        }

        fn ignore_side(&self) -> IgnoreSide {
            IgnoreSide::None
        }

        fn side(&self) -> Option<Side> {
            None
        }

        fn collision_ignorable(&self) -> bool {
            true
        }
    }

    impl Material for MyIgnorableTile {
        fn kind(&self) -> Option<MaterialKind> {
            None
        }
    }

    struct MyInterceptor(TileXy, f32, f32);

    impl Physic for MyInterceptor {
        fn position(&self, w: &WorldConfig) -> WorldVec3 {
            self.0.into_(w)
        }

        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            static EMPTY: Vec<Force> = vec![];
            &EMPTY
        }

        fn volumes(
            &self,
            ref_: WorldVec3,
            _: bool,
            w: &WorldConfig,
            _: &Mod,
        ) -> Vec<(Volume, Traversability, Direction)> {
            vec![(
                Volume::Cube {
                    x: ref_.x,
                    y: ref_.y,
                    z: ref_.z,
                    width: w.geo_pixels_per_tile() as f32,
                    height: w.geo_pixels_per_tile() as f32,
                    depth: self.2,
                },
                Traversability::all(),
                Direction::IDENTITY,
            )]
        }

        fn ignore_side(&self) -> IgnoreSide {
            IgnoreSide::None
        }

        fn intercept(&self, _kind: MaterialKind, _mod_: &Mod) -> f32 {
            self.1
        }
    }

    impl Material for MyInterceptor {
        fn kind(&self) -> Option<MaterialKind> {
            None
        }
    }

    type StepResult = (WorldVec3, Vec<Force>, Vec<Event<MyObjectId>>);

    fn load_mod() -> Mod {
        Mod::load(&workspace_root().join("mods/tests1"), None).unwrap()
    }

    fn world(size: u64, tick_hz: f32) -> WorldConfig {
        WorldConfig::new(size, size)
            .with_physics_tick(Frequency::new(tick_hz))
            .with_geo_pixels_per_meters(10.)
    }

    fn translation(direction: [f32; 3], speed: f32) -> Force {
        Force::Translation(direction.into(), MetersSeconds(speed))
    }

    fn moving_object(position: [f32; 3], force: Force) -> MyObject {
        MyObject(position.into(), vec![force])
    }

    /// Run one step of `object` (id 0), without proximity and with `collision_at` objects.
    fn run_step<'a>(
        w: &WorldConfig,
        object: &'a MyObject,
        collision_at: impl Fn(Xy) -> Vec<(MyObjectId, &'a dyn Physic)>,
        collision_ignore_pixels: usize,
    ) -> StepResult {
        step(
            w,
            w.physics_tick().period(),
            &load_mod(),
            (MyObjectId(0), object),
            collision_at,
            |_| vec![],
            collision_ignore_pixels,
            "test",
        )
    }

    /// Run one step of `object` (id 0) alone in the world.
    fn run_free_step(w: &WorldConfig, object: &MyObject) -> StepResult {
        run_step(w, object, |_| vec![], 0)
    }

    /// Tile 0 (traversable, id 1), then tile 1 (solid, id 2) and further.
    fn traversable_then_solid<'a>(
        traversable: &'a dyn Physic,
        solid: &'a dyn Physic,
    ) -> impl Fn(Xy) -> Vec<(MyObjectId, &'a dyn Physic)> {
        move |xy| {
            if xy == Xy(0, 0) {
                vec![(MyObjectId(1), traversable)]
            } else {
                vec![(MyObjectId(2), solid)]
            }
        }
    }

    /// Only `other` (id 1), and only on tile 0,0.
    fn only_on_first_tile<'a>(
        other: &'a dyn Physic,
    ) -> impl Fn(Xy) -> Vec<(MyObjectId, &'a dyn Physic)> {
        move |xy| {
            if xy == Xy(0, 0) {
                vec![(MyObjectId(1), other)]
            } else {
                vec![]
            }
        }
    }

    #[test]
    fn test_unidirectional_translation_x() {
        // Given
        let w = world(1000, 2.0);
        let object = moving_object([0.0, 0.0, 0.0], translation([1.0, 0.0, 0.0], 1.0));

        // When
        let (new_position, _, _) = run_free_step(&w, &object);

        // Then
        let expected_new_position: WorldVec3 = [5.0, 0.0, 0.0].into();
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_unidirectional_translation_outside() {
        // Given
        let w = world(10, 1.0);
        let object = moving_object([0.0, 0.0, 0.0], translation([1.0, 0.0, 0.0], 100.0));

        // When
        let (new_position, _, events) = run_free_step(&w, &object);

        // Then
        let expected_new_position: WorldVec3 = [49.0, 0.0, 0.0].into();
        assert_eq!(new_position, expected_new_position);
        assert_eq!(events, vec![Event::NoTile(MyObjectId(0))]);
    }

    #[test]
    fn test_unidirectional_translation_multisteps() {
        // Given
        let w = world(1000, 1.0);
        // 1% of 10 pixels = 0.1 pixel
        let object = moving_object([0.0, 0.0, 0.0], translation([1.0, 0.0, 0.0], 0.01));

        // When
        let (new_position, _, _) = run_free_step(&w, &object);

        // Then
        let expected_new_x = "0.1"; // step must complete bresenham pixel (which are isize) with end (force) position
        assert_eq!(&format!("{:.01}", new_position.x), expected_new_x);
    }

    #[test]
    fn test_unidirectional_translation_collision() {
        // Given
        let w = world(1000, 2.0);
        let object = moving_object([0.0, 0.0, 0.0], translation([1.0, 0.0, 0.0], 100.0));
        let my_traversable_tile = MyTile(TileXy(Xy(0, 0)), Traversability::all());
        let my_solid_tile = MyTile(TileXy(Xy(1, 0)), Traversability::none());

        // When
        let (new_position, new_forces, _) = run_step(
            &w,
            &object,
            traversable_then_solid(&my_traversable_tile, &my_solid_tile),
            0,
        );

        // Then
        let expected_new_position: WorldVec3 = [5.0, 0.0, 0.0].into();
        assert_eq!(new_position, expected_new_position);
        assert_eq!(new_forces, Vec::<Force>::new());
    }

    #[test]
    fn test_unidirectional_translation_high_speed() {
        // Given
        let w = world(1000, 1.0);
        let object = moving_object([0.0, 0.0, 0.0], translation([1.0, 0.0, 0.0], 10.0));

        // When
        let (new_position, _, _) = run_free_step(&w, &object);

        // Then
        let expected_new_position: WorldVec3 = [100.0, 0.0, 0.0].into();
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_unidirectional_translation_high_speed_collision() {
        // Given
        let w = world(1000, 1.0);
        let object = moving_object([0.0, 0.0, 0.0], translation([1.0, 0.0, 0.0], 100.0));
        let my_traversable_tile = MyTile(TileXy(Xy(0, 0)), Traversability::all());
        let my_solid_tile = MyTile(TileXy(Xy(1, 0)), Traversability::none());

        // When
        let (new_position, new_forces, _) = run_step(
            &w,
            &object,
            traversable_then_solid(&my_traversable_tile, &my_solid_tile),
            0,
        );

        // Then
        let expected_new_position: WorldVec3 = [5., 0.0, 0.0].into();
        assert_eq!(new_position, expected_new_position);
        assert_eq!(new_forces, Vec::<Force>::new());
    }

    #[test]
    fn test_bidirectional_translation() {
        // Given
        let w = world(1000, 1.0);
        let object = moving_object([0.0, 0.0, 0.0], translation([1.0, 1.0, 0.0], 1.0));

        // When
        let (new_position, _, _) = run_free_step(&w, &object);

        // Then
        let expected_new_position: WorldVec3 = [10.0, 10.0, 0.0].into();
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_collision_with_volume_on_same_tile_centered() {
        // Given
        let w = world(1000, 1.0).with_geo_pixels_per_tile(5);
        let object = moving_object([7.0, 2.0, 5.0], translation([-1.0, 0.0, 0.0], 1.0));
        let individual = MyIndividual([2.0, 2.0, 0.0].into()); // MyIndividual volume is 2 px !

        // When: expect collision when on tile 0,0
        let (position, forces, events) = run_step(&w, &object, only_on_first_tile(&individual), 0);

        // Then
        let expected_position: WorldVec3 = [4.0, 2.0, 5.0].into();
        assert_eq!(position, expected_position); // x axis hit first, at MySubject (x) 2.0 (position) + 2.0 (width)
        assert_eq!(forces, Vec::<Force>::new());
        assert_eq!(
            events,
            vec![Event::Collision(
                MyObjectId(0),
                MyObjectId(1),
                TickProgress(0.)
            )]
        );
    }

    #[test]
    fn test_collision_with_volume_on_same_tile_decal() {
        // Given
        let w = world(1000, 1.0).with_geo_pixels_per_tile(5);
        // Tile 1 (.5) on x; tile 0 (0.5) on y
        let object = moving_object([7.5, 2.5, 5.0], translation([-1.0, 0.0, 0.0], 1.0));
        // MyIndividual volume size (see MyIndividual Physics impl) should be impacted
        let individual = MyIndividual([1.0, 1.0, 0.0].into());

        // When: expect collision when on tile 0,0
        let (position, forces, events) = run_step(&w, &object, only_on_first_tile(&individual), 0);

        // Then
        let expected_position: WorldVec3 = [3.0, 3.0, 5.0].into();
        assert_eq!(position, expected_position);
        assert_eq!(forces, Vec::<Force>::new());
        assert_eq!(
            events,
            vec![Event::Collision(
                MyObjectId(0),
                MyObjectId(1),
                TickProgress(0.)
            )]
        );
    }

    #[test]
    fn test_unidirectional_translation_collision_ignore() {
        // Given
        let w = world(1000, 1.0);
        let object = moving_object([0.0, 0.0, 0.0], translation([1.0, 0.0, 0.0], 100.0));
        // Tile 0 is solid but ignorable (should be crossed thanks to collision_ignore)
        let my_ignorable_solid_tile = MyIgnorableTile(TileXy(Xy(0, 0)), Traversability::none());
        // Tile 1 is solid and NOT ignorable (should still block the object)
        let my_solid_tile = MyTile(TileXy(Xy(1, 0)), Traversability::none());

        // When: collision_ignore covers every pixel of tile 0, so its (ignorable) collisions
        // are all skipped and the object only stops when it reaches the non-ignorable tile 1
        let (new_position, new_forces, events) = run_step(
            &w,
            &object,
            traversable_then_solid(&my_ignorable_solid_tile, &my_solid_tile),
            5,
        );

        // Then
        let expected_new_position: WorldVec3 = [5.0, 0.0, 0.0].into();
        assert_eq!(new_position, expected_new_position);
        assert_eq!(new_forces, Vec::<Force>::new());
        assert_eq!(
            events,
            vec![Event::Collision(
                MyObjectId(0),
                MyObjectId(2),
                TickProgress(0.)
            )]
        );
    }

    fn run_intercept(intercept: f32) -> StepResult {
        run_intercept_at(intercept, 0.0, f32::MAX)
    }

    fn run_intercept_at(intercept: f32, object_z: f32, interceptor_depth: f32) -> StepResult {
        let w = world(1000, 2.0);
        let object = moving_object([0.0, 0.0, object_z], translation([1.0, 0.0, 0.0], 100.0));
        let interceptor = MyInterceptor(TileXy(Xy(0, 0)), intercept, interceptor_depth);

        run_step(
            &w,
            &object,
            |_| vec![(MyObjectId(1), &interceptor as &dyn Physic)],
            0,
        )
    }

    #[test]
    fn test_intercept_rolled_once_per_tile() {
        // Given: an interceptor of probability 0.5 on tile 0 only (crossed on several pixels)
        let w = world(1000, 2.0);
        assert!(w.geo_pixels_per_tile() > 1);
        let interceptor = MyInterceptor(TileXy(Xy(0, 0)), 0.5, f32::MAX);
        let object = moving_object([0.0, 0.0, 0.0], translation([1.0, 0.0, 0.0], 100.0));
        let collision_at = |xy: Xy| match xy == Xy(0, 0) {
            true => vec![(MyObjectId(1), &interceptor as &dyn Physic)],
            false => vec![],
        };
        fastrand::seed(42);

        // When
        let runs = 2000;
        let intercepted = (0..runs)
            .filter(|_| {
                let (_, _, events) = run_step(&w, &object, collision_at, 0);
                !events.is_empty()
            })
            .count();

        // Then: about half of objects are intercepted (not 1 - 0.5^pixels)
        let ratio = intercepted as f32 / runs as f32;
        assert!((0.45..0.55).contains(&ratio), "ratio {ratio}");
    }

    #[test]
    fn test_intercept_certain_stops_object() {
        // Given / When
        let (new_position, new_forces, events) = run_intercept(1.0);

        // Then
        assert!(new_position.x < 5.0);
        assert_eq!(new_forces, Vec::<Force>::new());
        assert_eq!(
            events,
            vec![Event::Collision(
                MyObjectId(0),
                MyObjectId(1),
                TickProgress(0.)
            )]
        );
    }

    #[test]
    fn test_intercept_zero_lets_object_pass() {
        // Given / When: no intercept and traversable volume
        let (new_position, _, events) = run_intercept(0.0);

        // Then: object reaches its destination without any collision
        let expected_new_position: WorldVec3 = [500.0, 0.0, 0.0].into();
        assert_eq!(new_position, expected_new_position);
        assert_eq!(events, vec![]);
    }

    #[test]
    fn test_intercept_ignored_when_over_volume_z() {
        // Given / When
        let (new_position, _, events) = run_intercept_at(1.0, 20.0, 10.0);

        // Then: object passes without any collision
        let expected_new_position: WorldVec3 = [500.0, 0.0, 20.0].into();
        assert_eq!(new_position, expected_new_position);
        assert_eq!(events, vec![]);
    }

    #[test]
    fn test_intercept_applied_when_in_volume_z() {
        // Given / When
        let (_, _, events) = run_intercept_at(1.0, 5.0, 10.0);

        // Then
        assert_eq!(
            events,
            vec![Event::Collision(
                MyObjectId(0),
                MyObjectId(1),
                TickProgress(0.)
            )]
        );
    }
}
