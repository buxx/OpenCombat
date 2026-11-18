use glam::Vec3;
// FIXME BS NOW: use fastrand ?
use rand::RngExt;

// WARNING: AI generated method
/// Computes a shooting direction from start to end, with inaccuracy applied.
///
/// `spread` is the maximum deviation angle (in degrees) from the
/// perfect aim direction. 0.0 = perfect accuracy, higher = less accurate.
/// A soldier who is bad at aiming might have spread = 5.0-15.0,
/// while a sniper might use 0.1-1.0.
pub fn direction_with_inaccuracy(original: Vec3, spread: f32) -> Vec3 {
    if spread <= 0.0 || original == Vec3::ZERO {
        return original;
    }

    let mut rng = rand::rng();

    // Build an orthonormal basis around perfect_direction so we can
    // rotate it randomly within a cone.
    let arbitrary = if original.x.abs() < 0.99 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let tangent = original.cross(arbitrary).normalize();
    let bitangent = original.cross(tangent);

    let max_angle_rad = spread.to_radians();

    // Uniform sampling over the spherical cap defined by max_angle_rad.
    // Using cos(theta) uniformly distributed gives uniform area sampling
    // on the cap (avoids bunching near the center or edge).
    let range = (max_angle_rad.cos())..1.0;
    if range.is_empty() {
        return original;
    }
    let cos_theta = rng.random_range(range);
    let theta = cos_theta.acos();
    let phi = rng.random_range(0.0..std::f32::consts::TAU);

    let sin_theta = theta.sin();
    let random_offset = tangent * (sin_theta * phi.cos())
        + bitangent * (sin_theta * phi.sin())
        + original * cos_theta;

    random_offset.normalize_or_zero()
}

/// Like [`direction_with_inaccuracy`], but the downward part of the deviation (pitch lower
/// than `original` one) is multiplied by `down_factor` (0.0 = never below aim, 1.0 = same as
/// [`direction_with_inaccuracy`]). Horizontal deviation is unchanged.
pub fn direction_with_inaccuracy_down(original: Vec3, spread: f32, down_factor: f32) -> Vec3 {
    let deviated = direction_with_inaccuracy(original, spread);
    let original_ = original.normalize_or_zero();
    let pitch = |d: Vec3| d.z.clamp(-1.0, 1.0).asin();
    let (from, to) = (pitch(original_), pitch(deviated));
    if to >= from {
        return deviated;
    }

    let horizontal = glam::Vec2::new(deviated.x, deviated.y).normalize_or_zero();
    if horizontal == glam::Vec2::ZERO {
        return deviated;
    }
    let to = from + (to - from) * down_factor.clamp(0.0, 1.0);
    Vec3::new(horizontal.x * to.cos(), horizontal.y * to.cos(), to.sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn down_deviation_reduced() {
        let original = Vec3::new(1.0, 0.0, -0.01).normalize();
        let pitch = original.z.asin();
        let spread = 5.0_f32.to_radians();
        for _ in 0..1000 {
            let d = direction_with_inaccuracy_down(original, 5.0, 0.25);
            assert!((d.length() - 1.0).abs() < 1e-4);
            // Never more than a quarter of the spread below the aim
            assert!(d.z.asin() >= pitch - spread * 0.25 - 1e-4);
            // Up deviation unchanged
            assert!(d.z.asin() <= pitch + spread + 1e-4);
        }
    }
}
