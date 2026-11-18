use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_mod::Mod;
use oc_root::{WcfgFrom, WorldConfig, material::MaterialKind};
use oc_world::tile::Tile;
use polyanya::*;

// FIXME: maybe merge these two functions ?
// function inspired from crates/oc_world/src/navmesh.rs which is built by AI
pub fn navmesh(w: &WorldConfig, mod_: &Mod, tiles: Vec<(&WorldTileIndex, &Tile)>) -> Mesh {
    // 1. Define the outer boundary (the full walkable world)
    //    Points go counter-clockwise around the perimeter.
    let width = w.world_width_pixels() as f32;
    let height = w.world_height_pixels() as f32;
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

    // 2. Add each blocked tile as a rectangular obstacle.
    //    Points must be in clockwise order for obstacles.
    for (i, tile) in tiles {
        // FIXME: wrote for individual only for now
        let wall = mod_
            .nature(tile.nature)
            .traversability
            .deny(MaterialKind::Individual);

        if !wall {
            continue;
        }

        let xy = TileXy::from_(*i, w);
        let x = xy.0.0 as f32 * tile_size;
        let y = xy.0.1 as f32 * tile_size;

        triangulation.add_obstacle([
            [x, y].into(),
            [x, y + tile_size].into(),
            [x + tile_size, y + tile_size].into(),
            [x + tile_size, y].into(),
        ]);
    }

    // 3. Convert triangulation → Mesh, merge + bake for efficiency
    let mut mesh = triangulation.as_navmesh();
    mesh.merge_polygons(); // collapses triangles into bigger convex polygons
    mesh.bake(); // builds internal spatial index for fast queries

    mesh
}
