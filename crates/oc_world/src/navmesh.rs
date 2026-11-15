use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_mod::Mod;
use oc_root::{WcfgFrom, WorldConfig};
use oc_utils::d2::Xy;
use oc_utils::index::AtIndex;
use polyanya::*;

// function built by AI (claude), then modified by myself
pub fn navmesh(w: &WorldConfig, grid: &[bool]) -> Mesh {
    // 1. Define the outer boundary (the full walkable world)
    //    Points go counter-clockwise around the perimeter.
    let width = w.world_width_pixels() as f32;
    let cols = w.world_width();
    let height = w.world_height_pixels() as f32;
    let rows = w.world_width();
    let tile_size = w.geo_pixels_per_tile() as f32;

    let mut triangulation = Triangulation::from_outer_edges(&[
        [0.0, 0.0].into(),
        [width, 0.0].into(),
        [width, height].into(),
        [0.0, height].into(),
    ]);

    // FIXME: values in config (copied from crates/oc_world/src/navmesh.rs)
    // Keeps the path center at least 2.5px away from any wall edge
    triangulation.set_agent_radius(2.5);
    // For tile walls, rounded corners aren't needed — fewer segments = faster
    triangulation.set_agent_radius_segments(1);
    // merge nearly-collinear points, epsilon in pixels²
    triangulation.simplify(0.1);

    // 2. Add blocked tiles as rectangular obstacles. Adjacent blocked tiles are merged in
    //    rectangles first: far fewer obstacles to triangulate than one by tile.
    //    Points must be in clockwise order for obstacles.
    for (col, row, cols_count, rows_count) in blocked_rectangles(w, grid, cols, rows) {
        let x = col as f32 * tile_size;
        let y = row as f32 * tile_size;
        let (width, height) = (cols_count as f32 * tile_size, rows_count as f32 * tile_size);

        triangulation.add_obstacle([
            [x, y].into(),
            [x, y + height].into(),
            [x + width, y + height].into(),
            [x + width, y].into(),
        ]);
    }

    // 3. Convert triangulation → Mesh, merge + bake for efficiency
    let mut mesh = triangulation.as_navmesh();
    mesh.merge_polygons(); // collapses triangles into bigger convex polygons
    mesh.bake(); // builds internal spatial index for fast queries

    mesh
}

pub trait Walls {
    fn as_walls(&self, mod_: &Mod) -> Vec<bool>;
}

/// Cover blocked tiles of `grid` with rectangles `(col, row, cols_count, rows_count)`:
/// horizontal runs of blocked tiles, merged with identical runs of following rows.
fn blocked_rectangles(
    w: &WorldConfig,
    grid: &[bool],
    cols: u64,
    rows: u64,
) -> Vec<(u64, u64, u64, u64)> {
    let blocked = |col, row| *grid.at(WorldTileIndex::from_(TileXy(Xy(col, row)), w));

    let mut rectangles: Vec<(u64, u64, u64, u64)> = vec![];
    // Rectangles (index in `rectangles`) still extensible downward, by their first col
    let mut open: Vec<usize> = vec![];
    for row in 0..rows {
        let mut still_open = vec![];
        let mut col = 0;
        while col < cols {
            if !blocked(col, row) {
                col += 1;
                continue;
            }
            let start = col;
            while col < cols && blocked(col, row) {
                col += 1;
            }
            let count = col - start;

            let above = open.iter().copied().find(|k| {
                let (c, r, n, h) = rectangles[*k];
                c == start && n == count && r + h == row
            });
            match above {
                Some(k) => {
                    rectangles[k].3 += 1;
                    still_open.push(k);
                }
                None => {
                    rectangles.push((start, row, count, 1));
                    still_open.push(rectangles.len() - 1);
                }
            }
        }
        open = still_open;
    }

    rectangles
}
