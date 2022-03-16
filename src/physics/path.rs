use crate::{map::Map, types::*};
use pathfinding::prelude::{absdiff, astar};

pub fn find_path(map: &Map, from: &GridPoint, to: &GridPoint) -> Option<Vec<GridPoint>> {
    if !map.contains(from) || !map.contains(to) {
        return None;
    }

    match astar(
        from,
        |p| map.successors(p),
        |p| absdiff(p.x, to.x) + absdiff(p.y, to.y),
        |p| *p == *to,
    ) {
        None => None,
        Some(path) => Some(path.0),
    }
}
