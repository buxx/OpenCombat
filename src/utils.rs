use serde::{Deserialize, Serialize};
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

pub const DARK_MAGENTA: Color = Color {
    r: 0.5,
    g: 0.0,
    b: 0.5,
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

pub fn vehicle_board_from_soldiers_on_board(soldier_on_board: &SoldiersOnBoard) -> VehicleBoard {
    let mut vehicle_board = VehicleBoard::new();
    for (soldier_index, (vehicle_index, place)) in soldier_on_board {
        vehicle_board
            .entry(*vehicle_index)
            .or_insert(vec![])
            .push((place.clone(), *soldier_index));
    }
    vehicle_board
}

pub enum AngleWay {
    ClockWise,
    CounterClockWise,
}

pub fn short_angle_way(current: &Angle, target: &Angle) -> AngleWay {
    let a = target.0 - current.0;
    let b = a + 360.;
    let c = a - 360.;

    if a.abs() < b.abs() && a.abs() < c.abs() {
        if a < 0. {
            AngleWay::CounterClockWise
        } else {
            AngleWay::ClockWise
        }
    } else if b.abs() < a.abs() && b.abs() < c.abs() {
        if b < 0. {
            AngleWay::CounterClockWise
        } else {
            AngleWay::ClockWise
        }
    } else {
        if c < 0. {
            AngleWay::CounterClockWise
        } else {
            AngleWay::ClockWise
        }
    }
}

pub fn short_angle(current: &Angle, target: &Angle) -> Angle {
    let a = target.0 - current.0;
    let b = a + 360.;
    let c = a - 360.;

    if a.abs() < b.abs() && a.abs() < c.abs() {
        Angle(a)
    } else if b.abs() < a.abs() && b.abs() < c.abs() {
        Angle(b)
    } else {
        Angle(c)
    }
}

pub fn grid_points_for_square(center_point: &GridPoint, width: i32, height: i32) -> Vec<GridPoint> {
    let mut points = vec![];

    let start_x = center_point.x - (height / 2);
    let end_x = start_x + height;
    let start_y = center_point.y - (width / 2);
    let end_y = start_y + width;

    for x in start_x..end_x {
        for y in start_y..end_y {
            points.push(GridPoint::new(x, y))
        }
    }

    points
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DebugPoint {
    pub frame_i: u64,
    pub point: WorldPoint,
}
