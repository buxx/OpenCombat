use std::f32::consts::FRAC_PI_2;

use derive_more::Constructor;
use glam::{Quat, Vec2, Vec3};
use oc_root::{
    WcfgFrom,
    geo::{WorldVec2, WorldVec3},
    y::V,
};

use crate::number::almost_equal;

#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Xy(pub u64, pub u64);

impl From<Xy> for (u64, u64) {
    fn from(value: Xy) -> Self {
        (value.0, value.1)
    }
}

impl From<(isize, isize)> for Xy {
    fn from((x, y): (isize, isize)) -> Self {
        Self(x as u64, y as u64)
    }
}

impl From<(isize, isize, isize)> for Xy {
    fn from((x, y, _): (isize, isize, isize)) -> Self {
        Self(x as u64, y as u64)
    }
}

impl From<Vec3> for Xy {
    fn from(value: Vec3) -> Self {
        Self(value.x as u64, value.y as u64)
    }
}

impl WcfgFrom<(f32, f32)> for Xy {
    fn from_(value: (f32, f32), w: &oc_root::WorldConfig) -> Self {
        Self(
            value.0 as u64 / w.geo_pixels_per_tile(),
            value.1 as u64 / w.geo_pixels_per_tile(),
        )
    }
}

impl WcfgFrom<(isize, isize)> for Xy {
    fn from_(value: (isize, isize), w: &oc_root::WorldConfig) -> Self {
        Self(
            value.0 as u64 / w.geo_pixels_per_tile(),
            value.1 as u64 / w.geo_pixels_per_tile(),
        )
    }
}

impl WcfgFrom<(usize, usize)> for Xy {
    fn from_(value: (usize, usize), w: &oc_root::WorldConfig) -> Self {
        Self(
            value.0 as u64 / w.geo_pixels_per_tile(),
            value.1 as u64 / w.geo_pixels_per_tile(),
        )
    }
}

impl Xy {
    pub fn distance(self, other: Self) -> f32 {
        ((other.0 as f32 - self.0 as f32).powi(2) + (other.1 as f32 - self.1 as f32).powi(2)).sqrt()
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Angle(pub f32);

impl Angle {
    pub fn from_points(to_point: &Vec2, from_point: &Vec2) -> Self {
        Self(f32::atan2(to_point.y - from_point.y, to_point.x - from_point.x) + FRAC_PI_2)
    }

    pub fn from_degrees(degrees: f32) -> Self {
        Self(degrees * (std::f32::consts::PI / 180.0))
    }

    pub fn zero() -> Self {
        Self(0.)
    }
}

impl std::ops::Add for Angle {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    /// X coordinate of the left edge of the rect.
    pub x: f32,
    /// Y coordinate of the top edge of the rect.
    pub y: f32,
    /// Total width of the rect
    pub w: f32,
    /// Total height of the rect.
    pub h: f32,
}

impl Rect {
    /// Create a new `Rect`.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.x, self.y, self.w, self.h]
    }

    pub fn from_array(values: [f32; 4]) -> Self {
        Self {
            x: values[0],
            y: values[1],
            w: values[2],
            h: values[3],
        }
    }
}

pub struct Shape {
    pub top_left: Vec2,
    pub top_right: Vec2,
    pub bottom_right: Vec2,
    pub bottom_left: Vec2,
}

impl Shape {
    pub fn from_rect(rect: &Rect) -> Self {
        Self {
            top_left: Vec2::new(rect.x, rect.y),
            top_right: Vec2::new(rect.x + rect.w, rect.y),
            bottom_right: Vec2::new(rect.x + rect.w, rect.y + rect.h),
            bottom_left: Vec2::new(rect.x, rect.y + rect.h),
        }
    }

    pub fn rotate(&self, angle: &Angle) -> Self {
        let width = self.top_right.x - self.top_left.x;
        let height = self.bottom_left.y - self.top_left.y;
        let center_offset = Vec2::new(width / 2., height / 2.);
        let reference_point = self.top_left + center_offset;

        let (sin, cos) = f32::sin_cos(angle.0);
        let after_top_left = rotate_point(&self.top_left, &reference_point, sin, cos);
        let after_top_right = rotate_point(&self.top_right, &reference_point, sin, cos);
        let after_bottom_right = rotate_point(&self.bottom_right, &reference_point, sin, cos);
        let after_bottom_left = rotate_point(&self.bottom_left, &reference_point, sin, cos);

        Self {
            top_left: after_top_left,
            top_right: after_top_right,
            bottom_right: after_bottom_right,
            bottom_left: after_bottom_left,
        }
    }

    pub fn from_point(&self, point: Vec2) -> Self {
        let width = self.top_right.x - self.top_left.x;
        let height = self.bottom_left.y - self.top_left.y;

        Self {
            top_left: point,
            top_right: point + Vec2::new(width, 0.),
            bottom_right: point + Vec2::new(width, height),
            bottom_left: point + Vec2::new(0., height),
        }
    }

    pub fn centered(&self) -> Self {
        let width = self.top_right.x - self.top_left.x;
        let height = self.bottom_left.y - self.top_left.y;

        Self {
            top_left: self.top_left - Vec2::new(width / 2., height / 2.),
            top_right: self.top_right - Vec2::new(width / 2., height / 2.),
            bottom_right: self.bottom_right - Vec2::new(width / 2., height / 2.),
            bottom_left: self.bottom_left - Vec2::new(width / 2., height / 2.),
        }
    }

    // WARNING: AI optimized code
    pub fn contains(&self, point: &Vec2) -> bool {
        // Convex quad: point is inside when it is on the same side of the four edges
        let corners = [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ];
        let (mut positive, mut negative) = (false, false);
        for k in 0..4 {
            let (a, b) = (corners[k], corners[(k + 1) % 4]);
            let cross = (b.x - a.x) * (point.y - a.y) - (b.y - a.y) * (point.x - a.x);
            positive |= cross > 0.;
            negative |= cross < 0.;
            if cross == 0. || (positive && negative) {
                return false;
            }
        }
        true
    }
}

pub fn apply_angle_on_point(point_to_rotate: &Vec2, reference_point: &Vec2, angle: &Angle) -> Vec2 {
    let (sin, cos) = f32::sin_cos(angle.0);
    rotate_point(point_to_rotate, reference_point, sin, cos)
}

#[inline]
fn rotate_point(point_to_rotate: &Vec2, reference_point: &Vec2, sin: f32, cos: f32) -> Vec2 {
    let pt = (
        point_to_rotate.x - reference_point.x,
        point_to_rotate.y - reference_point.y,
    );
    let rotated = (
        reference_point.x + pt.0 * cos - pt.1 * sin,
        reference_point.y + pt.0 * sin + pt.1 * cos,
    );
    Vec2::new(rotated.0, rotated.1)
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Constructor)]
pub struct Direction {
    pub x: f32,
    pub y: f32,
}

impl Default for Direction {
    fn default() -> Self {
        Self::NORTH
    }
}

impl Direction {
    pub const NORTH: Self = Self::new(0., 1.);
    pub const EST: Self = Self::new(1., 0.);
    pub const SOUTH: Self = Self::new(0., -1.);
    pub const WEST: Self = Self::new(-1., 0.);

    pub fn from_points2d(a: Vec2, b: Vec2) -> Self {
        Self::from((b - a).normalize_or_zero())
    }

    pub fn from_points3d(a: Vec3, b: Vec3) -> Self {
        Self::from((b - a).normalize_or_zero())
    }

    pub fn angle(&self, v: V) -> Angle {
        let y = match v {
            V::Server => self.y,
            V::Gui => -self.y,
        };
        Angle(y.atan2(self.x) - std::f32::consts::FRAC_PI_2)
    }

    #[cfg(feature = "bevy")]
    pub fn bquat(&self, v: V) -> bevy::prelude::Quat {
        let angle = self.angle(v);
        bevy::prelude::Quat::from_rotation_z(angle.0)
    }

    // WARNING: AI optimized code !
    /// Same rotation than `Quat::from_rotation_z(self.angle(v).0)`, without trigonometry.
    /// (Quaternion sign can differ: it is the same rotation.)
    pub fn quat(&self, v: V) -> Quat {
        let y = match v {
            V::Server => self.y,
            V::Gui => -self.y,
        };
        let length_squared = self.x * self.x + y * y;
        if length_squared < 1e-12 {
            return Quat::from_rotation_z(self.angle(v).0);
        }

        // Angle is atan2(y, x) - PI/2: so its cos is y and its sin is -x (once normalized)
        let inv_length = length_squared.sqrt().recip();
        let (cos, sin) = (y * inv_length, -self.x * inv_length);
        // Half angle, using the formula which doesn't lose precision
        let (half_sin, half_cos) = if cos >= 0. {
            let half_cos = ((1. + cos) * 0.5).sqrt();
            (sin / (2. * half_cos), half_cos)
        } else {
            let half_sin = ((1. - cos) * 0.5).sqrt().copysign(sin);
            (half_sin, sin / (2. * half_sin))
        };
        Quat::from_xyzw(0., 0., half_sin, half_cos)
    }

    // WARNING: AI generated
    /// Signed angle (radians, in `[-PI, PI]`) to rotate from this direction to `other`
    pub fn angle_to(&self, other: &Direction) -> f32 {
        let diff = other.y.atan2(other.x) - self.y.atan2(self.x);
        (diff + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
    }

    // WARNING: AI generated
    /// This direction rotated toward `target` by at most `max` radians (`target` when reachable)
    pub fn rotated_toward(&self, target: &Direction, max: f32) -> Direction {
        let angle = self.angle_to(target);
        if angle.abs() <= max {
            return *target;
        }

        let angle = self.y.atan2(self.x) + max.copysign(angle);
        Direction::new(angle.cos(), angle.sin())
    }

    /// Rotate a vector (vx, vy) by this direction, treating
    /// (x, y) as (cos θ, sin θ) of the rotation.
    #[inline]
    pub fn rotate(&self, vx: f32, vy: f32) -> (f32, f32) {
        (vx * self.x - vy * self.y, vx * self.y + vy * self.x)
    }
}

impl From<Direction> for Vec2 {
    fn from(value: Direction) -> Self {
        Vec2::new(value.x, value.y)
    }
}

impl From<Vec2> for Direction {
    fn from(value: Vec2) -> Self {
        Direction::new(value.x, value.y)
    }
}

impl From<WorldVec2> for Direction {
    fn from(value: WorldVec2) -> Self {
        Direction::new(value.x, value.y)
    }
}

impl From<Vec3> for Direction {
    fn from(value: Vec3) -> Self {
        Direction::new(value.x, value.y)
    }
}

impl From<WorldVec3> for Direction {
    fn from(value: WorldVec3) -> Self {
        Direction::new(value.x, value.y)
    }
}

// Function content IA generated
pub fn shape_cover_tiles(
    point: [f32; 2],
    width: f32,
    height: f32,
    tile_width: f32,
    tile_height: f32,
) -> Vec<[isize; 2]> {
    #[inline]
    fn range_indices(pos: f32, size: f32, tile: f32) -> (isize, isize) {
        let min_idx = (pos / tile) as isize;
        let end = pos + size;
        let max_idx = (end / tile).ceil() as isize - 1;
        (min_idx, max_idx)
    }

    let (col_min, col_max) = range_indices(point[0], width, tile_width);
    let (row_min, row_max) = range_indices(point[1], height, tile_height);

    let count = ((col_max - col_min + 1).max(0) * (row_max - row_min + 1).max(0)) as usize;
    let mut result = Vec::with_capacity(count);
    for row in row_min..=row_max {
        for col in col_min..=col_max {
            result.push([col, row]);
        }
    }
    result
}

pub trait AlmostEqual: Sized {
    fn almost_equal(&self, other: impl Into<Self>, tolerance: f32) -> bool;
}

impl AlmostEqual for WorldVec2 {
    fn almost_equal(&self, other: impl Into<WorldVec2>, tolerance: f32) -> bool {
        let other = other.into();
        almost_equal(self.x, other.x, tolerance) && almost_equal(self.y, other.y, tolerance)
    }
}

impl AlmostEqual for WorldVec3 {
    fn almost_equal(&self, other: impl Into<WorldVec3>, tolerance: f32) -> bool {
        let other = other.into();
        almost_equal(self.x, other.x, tolerance)
            && almost_equal(self.y, other.y, tolerance)
            && almost_equal(self.z, other.z, tolerance)
    }
}

#[cfg(test)]
mod tests {

    // WARNING: AI generated
    #[test]
    fn test_direction_angle_to() {
        assert!(almost_equal(
            Direction::NORTH.angle_to(&Direction::NORTH),
            0.,
            1e-5
        ));
        assert!(almost_equal(
            Direction::EST.angle_to(&Direction::NORTH),
            FRAC_PI_2,
            1e-5
        ));
        assert!(almost_equal(
            Direction::NORTH.angle_to(&Direction::EST),
            -FRAC_PI_2,
            1e-5
        ));
        // Shortest way, across PI
        let a = Direction::new(-1., 0.1);
        let b = Direction::new(-1., -0.1);
        assert!(a.angle_to(&b) > 0. && a.angle_to(&b) < 0.3);
    }

    // WARNING: AI generated
    #[test]
    fn test_direction_rotated_toward() {
        // Target reachable
        let d = Direction::EST.rotated_toward(&Direction::NORTH, FRAC_PI_2 + 0.1);
        assert_eq!(d, Direction::NORTH);

        // Target not reachable: rotate of max
        let d = Direction::EST.rotated_toward(&Direction::NORTH, FRAC_PI_2 / 2.);
        assert!(almost_equal(d.x, d.y, 1e-5));
        assert!(d.x > 0.);

        // Rotate by the shortest way
        let d = Direction::NORTH.rotated_toward(&Direction::EST, 0.1);
        assert!(d.x > 0. && d.y > 0.);
    }
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0., 1.)]
    #[case(1., 0.)]
    #[case(0., -1.)]
    #[case(-1., 0.)]
    #[case(0.6, 0.8)]
    #[case(-0.6, 0.8)]
    #[case(-0.6, -0.8)]
    #[case(0.6, -0.8)]
    #[case(3., 4.)]
    #[case(0.001, 1.)]
    #[case(0., 0.)]
    fn test_direction_quat_is_same_rotation_than_angle(#[case] x: f32, #[case] y: f32) {
        for v in [V::Server, V::Gui] {
            let direction = Direction::new(x, y);
            let expected = Quat::from_rotation_z(direction.angle(v).0);

            let quat = direction.quat(v);

            // q and -q are the same rotation
            assert!(
                quat.dot(expected).abs() > 1. - 1e-5,
                "{quat:?} != {expected:?}"
            );
        }
    }

    #[test]
    fn test_distance_line_x() {
        // Given
        let a = Xy(0, 0);
        let b = Xy(1, 0);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_distance_line_x_neg() {
        // Given
        let a = Xy(1, 0);
        let b = Xy(0, 0);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_distance_line_y() {
        // Given
        let a = Xy(0, 0);
        let b = Xy(0, 1);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_distance_diag() {
        // Given
        let a = Xy(0, 0);
        let b = Xy(1, 1);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, std::f32::consts::SQRT_2);
    }

    #[rstest]
    #[case([0., 0.], 2., 2., vec![[0, 0]])]
    #[case([4., 0.], 2., 2., vec![[0, 0], [1, 0]])]
    #[case([0., 4.], 2., 2., vec![[0, 0], [0, 1]])]
    #[case([4., 4.], 2., 2., vec![[0, 0], [1, 0], [0, 1], [1, 1]])]
    #[case([0., 0.], 10., 2., vec![[0, 0], [1, 0]])]
    #[case([0., 0.], 2., 10., vec![[0, 0], [0, 1]])]
    #[case([0., 0.], 10., 10., vec![[0, 0], [1, 0], [0, 1], [1, 1]])]
    fn test_shape_cover_tiles(
        #[case] point: [f32; 2],
        #[case] width: f32,
        #[case] height: f32,
        #[case] expected: Vec<[isize; 2]>,
    ) {
        let result = shape_cover_tiles(point, width, height, 5., 5.);
        assert_eq!(result, expected);
    }
}
