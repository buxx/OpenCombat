use std::{f32::consts::FRAC_PI_2, sync::atomic::AtomicUsize};

use ggez::graphics::Color;

use crate::types::*;

pub const GREEN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

pub const RED: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

pub const BLUE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

pub const YELLOW: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

pub const MAGENTA: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

pub fn new_squad_uuid() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub fn angle(to_point: &WorldPoint, from_point: &WorldPoint) -> Angle {
    // Note: angle computed by adding FRAC_PI_2 because sprites are north oriented
    Angle(f32::atan2(to_point.y - from_point.y, to_point.x - from_point.x) + FRAC_PI_2)
}
