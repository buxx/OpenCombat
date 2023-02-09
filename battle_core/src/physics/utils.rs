use crate::types::{Meters, WorldPoint};

// Coefficient to convert distance from two scene points into meters
pub const DISTANCE_TO_METERS_COEFFICIENT: f32 = 0.3;

pub fn meters_between_world_points(from: &WorldPoint, to: &WorldPoint) -> Meters {
    Meters(from.to_vec2().distance(to.to_vec2()) * DISTANCE_TO_METERS_COEFFICIENT)
}
