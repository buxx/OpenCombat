use geo::{coord, Contains, Triangle};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::{f32::consts::FRAC_PI_2, sync::atomic::AtomicUsize};

use crate::{physics::utils::DISTANCE_TO_METERS_COEFFICIENT, types::*};

pub fn new_squad_uuid() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub fn angle(to_point: &WorldPoint, from_point: &WorldPoint) -> Angle {
    // Note: angle computed by adding FRAC_PI_2 because sprites are north oriented
    Angle(f32::atan2(to_point.y - from_point.y, to_point.x - from_point.x) + FRAC_PI_2)
}

pub fn angleg(to_point: &GridPoint, from_point: &GridPoint) -> Angle {
    // Note: angle computed by adding FRAC_PI_2 because sprites are north oriented
    Angle(
        f32::atan2(
            to_point.y as f32 - from_point.y as f32,
            to_point.x as f32 - from_point.x as f32,
        ) + FRAC_PI_2,
    )
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

pub struct WorldShape {
    pub top_left: WorldPoint,
    pub top_right: WorldPoint,
    pub bottom_right: WorldPoint,
    pub bottom_left: WorldPoint,
}

impl WorldShape {
    pub fn from_rect(rect: &Rect) -> Self {
        Self {
            top_left: WorldPoint::new(rect.x, rect.y),
            top_right: WorldPoint::new(rect.x + rect.w, rect.y),
            bottom_right: WorldPoint::new(rect.x + rect.w, rect.y + rect.h),
            bottom_left: WorldPoint::new(rect.x, rect.y + rect.h),
        }
    }

    pub fn rotate(&self, angle: &Angle) -> Self {
        let width = self.top_right.x - self.top_left.x;
        let height = self.bottom_left.y - self.top_left.y;
        let center_offset = Vec2::new(width / 2., height / 2.);
        let reference_point = self.top_left.apply(center_offset);

        let after_top_left = apply_angle_on_point(&self.top_left, &reference_point, &angle);
        let after_top_right = apply_angle_on_point(&self.top_right, &reference_point, &angle);
        let after_bottom_right = apply_angle_on_point(&self.bottom_right, &reference_point, &angle);
        let after_bottom_left = apply_angle_on_point(&self.bottom_left, &reference_point, &angle);

        Self {
            top_left: after_top_left,
            top_right: after_top_right,
            bottom_right: after_bottom_right,
            bottom_left: after_bottom_left,
        }
    }

    pub fn cut(&self, selectable: Offset) -> Self {
        let top_right_x_len = self.top_right.x - self.top_left.x;
        let top_right_y_len = self.top_right.y - self.top_left.y;

        let bottom_right_x_len = self.bottom_right.x - self.top_right.x;
        let bottom_right_y_len = self.bottom_right.y - self.top_right.y;

        let new_top_left = self.top_left;
        let new_top_right = WorldPoint::new(
            self.top_left.x + top_right_x_len * selectable.x,
            self.top_left.y + top_right_y_len * selectable.x,
        );
        let new_bottom_right = WorldPoint::new(
            self.top_right.x + bottom_right_x_len * selectable.y,
            self.top_right.y + bottom_right_y_len * selectable.y,
        );
        let new_bottom_left = WorldPoint::new(
            self.top_left.x + bottom_right_x_len * selectable.y,
            self.top_left.y + bottom_right_y_len * selectable.y,
        );

        Self {
            top_left: new_top_left,
            top_right: new_top_right,
            bottom_right: new_bottom_right,
            bottom_left: new_bottom_left,
        }
    }

    pub fn from_meters(width: Meters, height: Meters) -> Self {
        let width_ = width.0 / DISTANCE_TO_METERS_COEFFICIENT;
        let height_ = height.0 / DISTANCE_TO_METERS_COEFFICIENT;

        Self {
            top_left: WorldPoint::new(0., 0.),
            top_right: WorldPoint::new(width_, 0.),
            bottom_right: WorldPoint::new(width_, height_),
            bottom_left: WorldPoint::new(0., height_),
        }
    }

    pub fn from_point(&self, point: WorldPoint) -> WorldShape {
        let width = self.top_right.x - self.top_left.x;
        let height = self.bottom_left.y - self.top_left.y;

        Self {
            top_left: point,
            top_right: point.apply(Vec2::new(width, 0.)),
            bottom_right: point.apply(Vec2::new(width, height)),
            bottom_left: point.apply(Vec2::new(0., height)),
        }
    }

    pub fn centered(&self) -> Self {
        let width = self.top_right.x - self.top_left.x;
        let height = self.bottom_left.y - self.top_left.y;

        Self {
            top_left: self.top_left.apply(-Vec2::new(width / 2., height / 2.)),
            top_right: self.top_right.apply(-Vec2::new(width / 2., height / 2.)),
            bottom_right: self.bottom_right.apply(-Vec2::new(width / 2., height / 2.)),
            bottom_left: self.bottom_left.apply(-Vec2::new(width / 2., height / 2.)),
        }
    }

    pub fn contains(&self, point: &WorldPoint) -> bool {
        let triangle1 = Triangle::new(
            coord! { x: self.top_left.x, y: self.top_left.y },
            coord! { x: self.top_right.x, y: self.top_right.y },
            coord! { x: self.bottom_left.x, y: self.bottom_left.y },
        );
        let triangle2 = Triangle::new(
            coord! { x: self.bottom_right.x, y: self.bottom_right.y },
            coord! { x: self.bottom_left.x, y: self.bottom_left.y },
            coord! { x: self.top_right.x, y: self.top_right.y },
        );

        triangle1.contains(&coord! { x: point.x, y: point.y })
            || triangle2.contains(&coord! { x: point.x, y: point.y })
    }
}

pub struct WindowShape {
    pub top_left: WindowPoint,
    pub top_right: WindowPoint,
    pub bottom_right: WindowPoint,
    pub bottom_left: WindowPoint,
}

impl WindowShape {
    pub fn draw_points(&self) -> Vec<Vec2> {
        vec![
            self.top_left.to_vec2(),
            self.top_right.to_vec2(),
            self.bottom_right.to_vec2(),
            self.bottom_left.to_vec2(),
            self.top_left.to_vec2(),
        ]
    }

    pub fn contains(&self, point: &WindowPoint) -> bool {
        let triangle1 = Triangle::new(
            coord! { x: self.top_left.x, y: self.top_left.y },
            coord! { x: self.top_right.x, y: self.top_right.y },
            coord! { x: self.bottom_left.x, y: self.bottom_left.y },
        );
        let triangle2 = Triangle::new(
            coord! { x: self.bottom_right.x, y: self.bottom_right.y },
            coord! { x: self.bottom_left.x, y: self.bottom_left.y },
            coord! { x: self.top_right.x, y: self.top_right.y },
        );

        triangle1.contains(&coord! { x: point.x, y: point.y })
            || triangle2.contains(&coord! { x: point.x, y: point.y })
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
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
