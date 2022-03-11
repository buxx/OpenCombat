use std::{f32::consts::FRAC_PI_2, sync::atomic::AtomicUsize};

use crate::types::*;

pub fn new_squad_uuid() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub fn angle(to_point: &WorldPoint, from_point: &WorldPoint) -> Angle {
    // Note: angle computed by adding FRAC_PI_2 because sprites are north oriented
    Angle(f32::atan2(to_point.y - from_point.y, to_point.x - from_point.x) + FRAC_PI_2)
}
