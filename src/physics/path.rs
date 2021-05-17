use crate::map::Map;
use crate::physics::GridPoint;
use pathfinding::prelude::{absdiff, astar};

pub fn find_path(map: &Map, from: &GridPoint, to: &GridPoint) -> Option<Vec<GridPoint>> {
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
