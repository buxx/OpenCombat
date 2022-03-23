use std::{f32::consts::FRAC_PI_2, sync::atomic::AtomicUsize};

use ggez::graphics::Color;
use glam::Vec2;

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

pub struct Rectangle<T> {
    pub top_left: T,
    pub top_right: T,
    pub bottom_left: T,
    pub bottom_right: T,
}

pub fn new_squad_uuid() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub fn angle(to_point: &WorldPoint, from_point: &WorldPoint) -> Angle {
    // Note: angle computed by adding FRAC_PI_2 because sprites are north oriented
    Angle(f32::atan2(to_point.y - from_point.y, to_point.x - from_point.x) + FRAC_PI_2)
}

pub fn apply_angle_on_point<T: Xy>(point_to_rotate: &T, reference_point: &T, angle: &Angle) -> T {
    let sin = f32::sin(angle.0);
    let cos = f32::cos(angle.0);
    let pt = (
        point_to_rotate.x() - reference_point.x(),
        point_to_rotate.y() - reference_point.y(),
    );
    let rotated = (
        reference_point.x() + pt.0 * cos - pt.1 * sin,
        reference_point.y() + pt.0 * sin + pt.1 * cos,
    );
    T::from_xy(rotated.0, rotated.1)
}
