use crate::{map::Map, types::*, utils::angleg};
use pathfinding::prelude::astar;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub enum PathMode {
    Walk,
    Drive(VehicleSize),
}
impl PathMode {
    pub fn include_vehicles(&self) -> bool {
        match self {
            PathMode::Walk => false,
            PathMode::Drive(_) => true,
        }
    }
}

pub const COST_AHEAD: i32 = 0;
pub const COST_DIAGONAL: i32 = 10;
pub const COST_CORNER: i32 = 20;
pub const COST_BACK_CORNER: i32 = 30;
pub const COST_BACK: i32 = 50;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter)]
pub enum Direction {
    North,
    NorthEst,
    Est,
    SouthEst,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    pub fn from_angle(angle: &Angle) -> Self {
        let degrees = angle.0.to_degrees();
        if degrees >= 337.5 || degrees <= 22.5 {
            Self::North
        } else if degrees > 22.5 && degrees <= 67.5 {
            Self::NorthEst
        } else if degrees > 67.5 && degrees <= 112.5 {
            Self::Est
        } else if degrees > 112.5 && degrees <= 157.5 {
            Self::SouthEst
        } else if degrees > 157.5 && degrees <= 202.5 {
            Self::South
        } else if degrees > 202.5 && degrees <= 247.5 {
            Self::SouthWest
        } else if degrees > 247.5 && degrees <= 292.5 {
            Self::West
        } else {
            Self::NorthWest
        }
    }

    pub fn modifier(&self) -> (i32, i32) {
        match self {
            Direction::NorthWest => (-1, -1),
            Direction::North => (0, -1),
            Direction::NorthEst => (1, -1),
            Direction::Est => (1, 0),
            Direction::SouthEst => (1, 1),
            Direction::South => (0, 1),
            Direction::SouthWest => (-1, 1),
            Direction::West => (0, -1),
        }
    }

    pub fn angle_cost(&self, direction: &Direction) -> i32 {
        match self {
            Direction::North => match direction {
                Direction::North => COST_AHEAD,
                Direction::NorthEst => COST_DIAGONAL,
                Direction::Est => COST_CORNER,
                Direction::SouthEst => COST_BACK_CORNER,
                Direction::South => COST_BACK,
                Direction::SouthWest => COST_BACK_CORNER,
                Direction::West => COST_CORNER,
                Direction::NorthWest => COST_DIAGONAL,
            },
            Direction::NorthEst => match direction {
                Direction::North => COST_DIAGONAL,
                Direction::NorthEst => COST_AHEAD,
                Direction::Est => COST_DIAGONAL,
                Direction::SouthEst => COST_CORNER,
                Direction::South => COST_BACK_CORNER,
                Direction::SouthWest => COST_BACK,
                Direction::West => COST_BACK_CORNER,
                Direction::NorthWest => COST_CORNER,
            },
            Direction::Est => match direction {
                Direction::North => COST_CORNER,
                Direction::NorthEst => COST_DIAGONAL,
                Direction::Est => COST_AHEAD,
                Direction::SouthEst => COST_DIAGONAL,
                Direction::South => COST_CORNER,
                Direction::SouthWest => COST_BACK_CORNER,
                Direction::West => COST_BACK,
                Direction::NorthWest => COST_BACK_CORNER,
            },
            Direction::SouthEst => match direction {
                Direction::North => COST_BACK_CORNER,
                Direction::NorthEst => COST_CORNER,
                Direction::Est => COST_DIAGONAL,
                Direction::SouthEst => COST_AHEAD,
                Direction::South => COST_DIAGONAL,
                Direction::SouthWest => COST_CORNER,
                Direction::West => COST_BACK_CORNER,
                Direction::NorthWest => COST_BACK,
            },
            Direction::South => match direction {
                Direction::North => COST_BACK,
                Direction::NorthEst => COST_BACK_CORNER,
                Direction::Est => COST_CORNER,
                Direction::SouthEst => COST_DIAGONAL,
                Direction::South => COST_AHEAD,
                Direction::SouthWest => COST_DIAGONAL,
                Direction::West => COST_CORNER,
                Direction::NorthWest => COST_BACK_CORNER,
            },
            Direction::SouthWest => match direction {
                Direction::North => COST_BACK_CORNER,
                Direction::NorthEst => COST_BACK,
                Direction::Est => COST_BACK_CORNER,
                Direction::SouthEst => COST_CORNER,
                Direction::South => COST_DIAGONAL,
                Direction::SouthWest => COST_AHEAD,
                Direction::West => COST_DIAGONAL,
                Direction::NorthWest => COST_CORNER,
            },
            Direction::West => match direction {
                Direction::North => COST_CORNER,
                Direction::NorthEst => COST_BACK_CORNER,
                Direction::Est => COST_BACK,
                Direction::SouthEst => COST_BACK_CORNER,
                Direction::South => COST_CORNER,
                Direction::SouthWest => COST_DIAGONAL,
                Direction::West => COST_AHEAD,
                Direction::NorthWest => COST_DIAGONAL,
            },
            Direction::NorthWest => match direction {
                Direction::North => COST_DIAGONAL,
                Direction::NorthEst => COST_CORNER,
                Direction::Est => COST_BACK_CORNER,
                Direction::SouthEst => COST_BACK,
                Direction::South => COST_BACK_CORNER,
                Direction::SouthWest => COST_CORNER,
                Direction::West => COST_DIAGONAL,
                Direction::NorthWest => COST_AHEAD,
            },
        }
    }
}

// TODO : When "to" is unreachable (ex. for vehicle) do not search a path (it consume all path before stop)
pub fn find_path(
    map: &Map,
    from: &GridPoint,
    to: &GridPoint,
    exclude_first: bool,
    path_mode: &PathMode,
    start_direction: &Option<Direction>,
) -> Option<Vec<GridPoint>> {
    if !map.contains(from) || !map.contains(to) {
        return None;
    }
    let start_direction = start_direction.unwrap_or(Direction::from_angle(&angleg(to, from)));

    match astar(
        &(*from, start_direction),
        |p| map.successors(p, path_mode),
        |p| (p.0.x.abs_diff(to.x) + p.0.y.abs_diff(to.y)) as i32,
        |p| p.0 == *to,
    ) {
        None => None,
        Some(path) => {
            if exclude_first {
                let new_path = path.0[1..].to_vec();
                if new_path.len() > 0 {
                    Some(new_path.iter().map(|x| x.0).collect())
                } else {
                    None
                }
            } else {
                Some(path.0.iter().map(|x| x.0).collect())
            }
        }
    }
}
