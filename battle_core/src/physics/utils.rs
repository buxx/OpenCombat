use crate::types::{Distance, WorldPoint};

// Coefficient to convert distance from two scene points into meters
// TODO : fix it with sprites, maps, etc
pub const DISTANCE_TO_METERS_COEFFICIENT: f32 = 0.3;

pub fn distance_between_points(from: &WorldPoint, to: &WorldPoint) -> Distance {
    Distance::from_millimeters(
        ((from.to_vec2().distance(to.to_vec2()) * DISTANCE_TO_METERS_COEFFICIENT) * 1000.) as i64,
    )
}
