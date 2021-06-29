use crate::behavior::ItemBehavior;
use crate::config::{MOVE_FAST_VELOCITY, MOVE_HIDE_VELOCITY, MOVE_VELOCITY};
use crate::{Angle, ScenePoint};
use ggez::graphics::Color;
use std::f32::consts::FRAC_PI_2;

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

pub struct Rectangle {
    pub top_left: ScenePoint,
    pub top_right: ScenePoint,
    pub bottom_left: ScenePoint,
    pub bottom_right: ScenePoint,
}

pub fn velocity_for_behavior(behavior: &ItemBehavior) -> Option<f32> {
    match behavior {
        ItemBehavior::MoveTo(_, _) => Some(MOVE_VELOCITY),
        ItemBehavior::MoveFastTo(_, _) => Some(MOVE_FAST_VELOCITY),
        ItemBehavior::HideTo(_, _) => Some(MOVE_HIDE_VELOCITY),
        _ => None,
    }
}

pub fn angle(to_point: &ScenePoint, from_point: &ScenePoint) -> Angle {
    f32::atan2(to_point.y - from_point.y, to_point.x - from_point.x) + FRAC_PI_2
}
