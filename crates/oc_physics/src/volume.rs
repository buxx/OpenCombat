use glam::{Quat, Vec3};

// WARNING: this module has been AI generated

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum Volume {
    Point {
        x: f32,
        y: f32,
        z: f32,
    },
    Cube {
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
        depth: f32,
    },
}

impl Volume {
    pub fn point_zero() -> Self {
        Self::Point {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }

    /// Repositions the volume so that `value` is its center (for `Cube`,
    /// `x/y/z` is stored as the min corner, so it is offset back by half the
    /// size on each axis; for `Point`, center and position are the same).
    pub fn with_ref(mut self, value: impl Into<[f32; 3]>) -> Self {
        let value: [f32; 3] = value.into();
        match &mut self {
            Volume::Point { x, y, z } => {
                *x = value[0];
                *y = value[1];
                *z = value[2];
            }
            Volume::Cube {
                x,
                y,
                z,
                width,
                height,
                depth: _,
            } => {
                *x = value[0] - *width / 2.0;
                *y = value[1] - *height / 2.0;
                *z = value[2];
            }
        };

        self
    }

    /// Whether `z` lies within the volume z extent (rotation ignored).
    pub fn contains_z(&self, z_: f32) -> bool {
        match self {
            Volume::Point { z, .. } => *z == z_,
            Volume::Cube { z, depth, .. } => z_ >= *z && z_ <= *z + *depth,
        }
    }

    /// Collision test with per-volume rotation.
    ///
    /// `self_rotation` / `other_rotation` rotate each volume about its own
    /// center (for `Cube`, `x/y/z` is treated as the min corner and the
    /// pivot is `pos + size / 2`; for `Point`, rotation has no effect since
    /// a dimensionless point can't change shape/position by spinning about
    /// itself).
    pub fn collide(&self, self_rotation: Quat, other: &Self, other_rotation: Quat) -> bool {
        match (self, other) {
            (
                Volume::Point {
                    x: x1,
                    y: y1,
                    z: z1,
                },
                Volume::Point {
                    x: x2,
                    y: y2,
                    z: z2,
                },
            ) => {
                // Rotation is irrelevant for point-vs-point: a point has no
                // extent, so spinning it about itself doesn't move it.
                x1 == x2 && y1 == y2 && z1 == z2
            }

            (
                Volume::Point {
                    x: px,
                    y: py,
                    z: pz,
                },
                Volume::Cube {
                    x: cx,
                    y: cy,
                    z: cz,
                    width,
                    height,
                    depth,
                },
            ) => point_in_rotated_cube(
                Vec3::new(*px, *py, *pz),
                Vec3::new(*cx, *cy, *cz),
                Vec3::new(*width, *height, *depth),
                other_rotation,
            ),

            (
                Volume::Cube {
                    x: cx,
                    y: cy,
                    z: cz,
                    width,
                    height,
                    depth,
                },
                Volume::Point {
                    x: px,
                    y: py,
                    z: pz,
                },
            ) => point_in_rotated_cube(
                Vec3::new(*px, *py, *pz),
                Vec3::new(*cx, *cy, *cz),
                Vec3::new(*width, *height, *depth),
                self_rotation,
            ),

            (
                Volume::Cube {
                    x: x1,
                    y: y1,
                    z: z1,
                    width: w1,
                    height: h1,
                    depth: d1,
                },
                Volume::Cube {
                    x: x2,
                    y: y2,
                    z: z2,
                    width: w2,
                    height: h2,
                    depth: d2,
                },
            ) => obb_collide(
                Vec3::new(*x1, *y1, *z1),
                Vec3::new(*w1, *h1, *d1),
                self_rotation,
                Vec3::new(*x2, *y2, *z2),
                Vec3::new(*w2, *h2, *d2),
                other_rotation,
            ),
        }
    }
}

/// Converts a min-corner + size cube into (center, half_extents).
fn cube_center_and_half_extents(pos: Vec3, size: Vec3) -> (Vec3, Vec3) {
    let half = size * 0.5;
    (pos + half, half)
}

/// Tests whether `point` lies inside a cube of `size` positioned at
/// min-corner `cube_pos` and rotated by `cube_rotation` about its own center.
fn point_in_rotated_cube(
    point: Vec3,
    cube_pos: Vec3,
    cube_size: Vec3,
    cube_rotation: Quat,
) -> bool {
    let (center, half_extents) = cube_center_and_half_extents(cube_pos, cube_size);

    // Move into the cube's local, unrotated space.
    let local_point = cube_rotation.inverse() * (point - center);

    local_point.x.abs() <= half_extents.x
        && local_point.y.abs() <= half_extents.y
        && local_point.z.abs() <= half_extents.z
}

/// Oriented bounding box collision via the Separating Axis Theorem (SAT).
///
/// Tests the 3 face normals of each box plus the 9 cross products of their
/// edge axes (15 axes total, the standard minimal set for 3D OBB-OBB SAT).
fn obb_collide(
    pos_a: Vec3,
    size_a: Vec3,
    rot_a: Quat,
    pos_b: Vec3,
    size_b: Vec3,
    rot_b: Quat,
) -> bool {
    let (center_a, half_a) = cube_center_and_half_extents(pos_a, size_a);
    let (center_b, half_b) = cube_center_and_half_extents(pos_b, size_b);

    let axes_a = [rot_a * Vec3::X, rot_a * Vec3::Y, rot_a * Vec3::Z];
    let axes_b = [rot_b * Vec3::X, rot_b * Vec3::Y, rot_b * Vec3::Z];

    let translation = center_b - center_a;

    let separated = |axis| is_separating_axis(axis, translation, &axes_a, half_a, &axes_b, half_b);

    // Face normals first, then edge cross products (computed lazily, only if still needed).
    if axes_a.iter().chain(&axes_b).any(|axis| separated(*axis)) {
        return false;
    }

    for a in &axes_a {
        for b in &axes_b {
            let cross = a.cross(*b);
            // Skip near-parallel edge pairs; their cross product is ~zero
            // and doesn't give a valid axis (already covered by face tests).
            if cross.length_squared() > 1e-6 && separated(cross.normalize()) {
                return false;
            }
        }
    }

    true
}

// Small tolerance so that boxes which are exactly face-to-face (or off by
// float rounding) are treated as touching-but-not-overlapping, matching the
// original strict `<`/`>` AABB semantics (touching == no collision).
const SAT_EPSILON: f32 = 1e-5;

fn is_separating_axis(
    axis: Vec3,
    translation: Vec3,
    axes_a: &[Vec3; 3],
    half_a: Vec3,
    axes_b: &[Vec3; 3],
    half_b: Vec3,
) -> bool {
    let proj_translation = translation.dot(axis).abs();

    let proj_a = half_a.x * axes_a[0].dot(axis).abs()
        + half_a.y * axes_a[1].dot(axis).abs()
        + half_a.z * axes_a[2].dot(axis).abs();

    let proj_b = half_b.x * axes_b[0].dot(axis).abs()
        + half_b.y * axes_b[1].dot(axis).abs()
        + half_b.z * axes_b[2].dot(axis).abs();

    proj_translation >= proj_a + proj_b - SAT_EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    // Point vs Point
    #[test]
    fn test_point_point_same() {
        let a = Volume::Point {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        let b = Volume::Point {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        assert!(a.collide(Quat::IDENTITY, &b, Quat::IDENTITY));
    }

    #[test]
    fn test_point_point_different() {
        let a = Volume::Point {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        let b = Volume::Point {
            x: 1.0,
            y: 2.0,
            z: 1.0,
        };
        assert!(!a.collide(Quat::IDENTITY, &b, Quat::IDENTITY));
    }

    // Point vs Cube
    #[test]
    fn test_point_inside_cube() {
        let p = Volume::Point {
            x: 2.0,
            y: 2.0,
            z: 2.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(p.collide(Quat::IDENTITY, &c, Quat::IDENTITY));
    }

    #[test]
    fn test_point_outside_cube() {
        let p = Volume::Point {
            x: 6.0,
            y: 6.0,
            z: 6.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!p.collide(Quat::IDENTITY, &c, Quat::IDENTITY));
    }

    #[test]
    fn test_point_outside_cube_z_axis() {
        let p = Volume::Point {
            x: 2.0,
            y: 2.0,
            z: 6.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!p.collide(Quat::IDENTITY, &c, Quat::IDENTITY));
    }

    #[test]
    fn test_point_on_cube_face() {
        let p = Volume::Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(p.collide(Quat::IDENTITY, &c, Quat::IDENTITY));
    }

    #[test]
    fn test_point_on_cube_corner() {
        let p = Volume::Point {
            x: 5.0,
            y: 5.0,
            z: 5.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(p.collide(Quat::IDENTITY, &c, Quat::IDENTITY));
    }

    // Symmetry: Cube vs Point should mirror Point vs Cube
    #[test]
    fn test_cube_point_symmetry() {
        let p = Volume::Point {
            x: 2.0,
            y: 2.0,
            z: 2.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert_eq!(
            p.collide(Quat::IDENTITY, &c, Quat::IDENTITY),
            c.collide(Quat::IDENTITY, &p, Quat::IDENTITY)
        );
    }

    // Cube vs Cube
    #[test]
    fn test_cubes_overlapping() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 3.0,
            y: 3.0,
            z: 3.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(a.collide(Quat::IDENTITY, &b, Quat::IDENTITY));
    }

    #[test]
    fn test_cubes_not_overlapping() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 6.0,
            y: 6.0,
            z: 6.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!a.collide(Quat::IDENTITY, &b, Quat::IDENTITY));
    }

    #[test]
    fn test_cubes_separated_on_z_axis() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 6.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!a.collide(Quat::IDENTITY, &b, Quat::IDENTITY));
    }

    #[test]
    fn test_cubes_touching_face() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 5.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!a.collide(Quat::IDENTITY, &b, Quat::IDENTITY));
    }

    #[test]
    fn test_cubes_one_inside_other() {
        let outer = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 10.0,
            height: 10.0,
            depth: 10.0,
        };
        let inner = Volume::Cube {
            x: 2.0,
            y: 2.0,
            z: 2.0,
            width: 3.0,
            height: 3.0,
            depth: 3.0,
        };
        assert!(outer.collide(Quat::IDENTITY, &inner, Quat::IDENTITY));
    }

    #[test]
    fn test_cubes_same_position() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(a.collide(Quat::IDENTITY, &b, Quat::IDENTITY));
    }

    #[test]
    fn test_cubes_overlap_symmetry() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 3.0,
            y: 3.0,
            z: 3.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert_eq!(
            a.collide(Quat::IDENTITY, &b, Quat::IDENTITY),
            b.collide(Quat::IDENTITY, &a, Quat::IDENTITY)
        );
    }
}
