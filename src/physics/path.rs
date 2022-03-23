use crate::{map::Map, types::*};
use pathfinding::prelude::{absdiff, astar};

pub fn find_path(
    map: &Map,
    from: &GridPoint,
    to: &GridPoint,
    exclude_first: bool,
) -> Option<Vec<GridPoint>> {
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
        Some(path) => {
            if exclude_first {
                let new_path = path.0[1..].to_vec();
                if new_path.len() > 0 {
                    Some(new_path)
                } else {
                    None
                }
            } else {
                Some(path.0)
            }
        }
    }
}
