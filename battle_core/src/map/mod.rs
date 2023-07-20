use std::path::PathBuf;

use self::{decor::Decor, interior::Interior, spawn::SpawnZone, terrain::TerrainTile};
use crate::{
    config::ServerConfig,
    game::{
        control::MapControl,
        flag::{Flag, FlagName},
    },
    physics::path::{Direction, PathMode},
    types::{GridPoint, VehicleSize, WorldPoint},
    utils::grid_points_for_square,
};
use oc_core::spawn::SpawnZoneName;
use strum::IntoEnumIterator;

pub mod decor;
pub mod interior;
pub mod reader;
pub mod spawn;
pub mod terrain;

#[derive(Clone)]
pub struct Map {
    name: String,
    background_image_path: PathBuf,
    interiors_image_path: PathBuf,
    terrain_image_path: PathBuf,
    interiors: Vec<Interior>,
    spawn_zones: Vec<SpawnZone>,
    width: u32,
    visual_width: u32,
    height: u32,
    visual_height: u32,
    terrain_tiles: Vec<TerrainTile>,
    tile_width: u32,
    tile_height: u32,
    decor: Decor,
    flags: Vec<Flag>,
}

impl Map {
    pub fn new(
        name: String,
        background_image_path: PathBuf,
        interiors_image_path: PathBuf,
        terrain_image_path: PathBuf,
        interiors: Vec<Interior>,
        spawn_zones: Vec<SpawnZone>,
        width: u32,
        height: u32,
        terrain_tiles: Vec<TerrainTile>,
        tile_width: u32,
        tile_height: u32,
        decor: Decor,
        flags: Vec<Flag>,
    ) -> Self {
        Self {
            name,
            background_image_path,
            interiors_image_path,
            terrain_image_path,
            interiors,
            spawn_zones,
            width,
            visual_width: width * tile_width,
            height,
            visual_height: height * tile_height,
            terrain_tiles,
            tile_width,
            tile_height,
            decor,
            flags,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn background_image_path(&self) -> &PathBuf {
        &self.background_image_path
    }

    pub fn interiors_image_path(&self) -> &PathBuf {
        &self.interiors_image_path
    }

    pub fn terrain_image_path(&self) -> &PathBuf {
        &self.terrain_image_path
    }

    pub fn interiors(&self) -> &Vec<Interior> {
        &self.interiors
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn visual_width(&self) -> u32 {
        self.visual_width
    }

    pub fn visual_height(&self) -> u32 {
        self.visual_height
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn decor(&self) -> &Decor {
        &self.decor
    }

    pub fn tile_width(&self) -> u32 {
        self.tile_width
    }

    pub fn tile_height(&self) -> u32 {
        self.tile_height
    }

    pub fn terrain_tiles(&self) -> &Vec<TerrainTile> {
        &self.terrain_tiles
    }

    pub fn flags(&self) -> &Vec<Flag> {
        &self.flags
    }

    // TODO : Get flags by name is not clean way, it could be better to use indexes (like soldiers, etc)
    pub fn flag(&self, flag_name: &FlagName) -> &Flag {
        self.flags()
            .iter()
            .filter(|f| f.name() == flag_name)
            .next()
            .expect("Flags ownership and map flag must be consistent")
    }

    pub fn find_spawn_zones(&self, names: &Vec<SpawnZoneName>) -> Vec<&SpawnZone> {
        self.spawn_zones
            .iter()
            .filter(|s| names.contains(&SpawnZoneName::All) || names.contains(&s.name()))
            .collect()
    }

    pub fn one_of_spawn_zone_contains_flag(
        &self,
        spawn_zone_names: &Vec<SpawnZoneName>,
        flag: &Flag,
    ) -> bool {
        for spawn_zone_name in spawn_zone_names {
            // FIXME BS NOW : algo moche ?!
            let foo = &vec![spawn_zone_name.clone()];
            let found = self.find_spawn_zones(foo);
            let spawn_zone = found.first().unwrap(); // FIXME BS NOW : manage error

            if spawn_zone.contains(&flag.shape()) {
                return true;
            }
        }

        return false;
    }

    pub fn successors(
        &self,
        from: &(GridPoint, Direction),
        path_mode: &PathMode,
    ) -> Vec<((GridPoint, Direction), i32)> {
        let mut successors = vec![];

        for direction in Direction::iter() {
            let (mod_x, mod_y) = direction.modifier();
            let new_x = from.0.x + mod_x;
            let new_y = from.0.y + mod_y;

            // Don't care ifd outside map
            if new_x < 0 || new_y < 0 || new_x > self.width as i32 || new_y > self.height as i32 {
                continue;
            }

            // If in map
            let i = (new_y * self.width() as i32 + new_x) as usize;
            if let Some(next_tile) = self.terrain_tiles.get(i) {
                if path_mode.include_vehicles() {
                    if next_tile.type_().block_vehicle() {
                        continue;
                    }

                    match path_mode {
                        PathMode::Drive(size) => {
                            if !self.point_allow_vehicle(&GridPoint::new(new_x, new_y), size) {
                                continue;
                            }
                        }
                        _ => {}
                    }
                }

                let cost = match path_mode {
                    PathMode::Walk => next_tile.type_().pedestrian_cost(),
                    PathMode::Drive(_size) => from.1.angle_cost(&direction),
                };

                successors.push(((GridPoint::new(new_x, new_y), direction), cost))
            }
        }

        successors
    }

    pub fn contains(&self, grid_point: &GridPoint) -> bool {
        grid_point.x >= 0
            && grid_point.y >= 0
            && grid_point.x < self.width() as i32
            && grid_point.y < self.height() as i32
    }

    pub fn grid_point_from_world_point(&self, world_point: &WorldPoint) -> GridPoint {
        let x = world_point.x as u32 / self.tile_width();
        let y = world_point.y as u32 / self.tile_height();
        GridPoint::new(x as i32, y as i32)
    }

    pub fn world_point_from_grid_point(&self, grid_point: GridPoint) -> WorldPoint {
        let x = (grid_point.x * self.tile_width() as i32) + (self.tile_width() as i32 / 2);
        let y = (grid_point.y * self.tile_height() as i32) + (self.tile_height() as i32 / 2);
        WorldPoint::new(x as f32, y as f32)
    }

    pub fn point_allow_vehicle(&self, point: &GridPoint, size: &VehicleSize) -> bool {
        let half = (size.0 / 2) as i32;
        let start_x = point.x - half;
        let end_x = point.x + half;
        let start_y = point.y - half;
        let end_y = point.y + half;

        for x in start_x..end_x {
            for y in start_y..end_y {
                if let Some(tile) = self
                    .terrain_tiles
                    .get((y * self.width() as i32 + x) as usize)
                {
                    if tile.type_().block_vehicle() {
                        return false;
                    }
                }
            }
        }

        return true;
    }

    pub fn point_in_spawn_zones(
        &self,
        point: &WorldPoint,
        allowed_zone_names: &MapControl,
        consider_all: bool,
    ) -> bool {
        if consider_all && allowed_zone_names.contains_spawn_zone(&SpawnZoneName::All) {
            return true;
        }

        for spawn_zone in &self.spawn_zones {
            if allowed_zone_names.contains_spawn_zone(spawn_zone.name()) {
                if point.x >= spawn_zone.x()
                    && point.x <= spawn_zone.x() + spawn_zone.width()
                    && point.y >= spawn_zone.y()
                    && point.y <= spawn_zone.y() + spawn_zone.height()
                {
                    return true;
                }
            }
        }

        false
    }
}

pub fn find_arbitrary_cover_grid_point(
    config: &ServerConfig,
    from_grid_point: &GridPoint,
    map: &Map,
    exclude_grid_points: &Vec<GridPoint>,
    cover_distance: i32,
) -> Option<(GridPoint, Vec<GridPoint>)> {
    let tiles = find_arbitrary_cover_grid_points(config, from_grid_point, map, cover_distance);
    for (grid_point, _) in tiles.iter().rev() {
        if !exclude_grid_points.contains(grid_point) {
            // TODO : This is for debug, don't take too much cpu ?
            let grid_points = tiles
                .iter()
                .map(|(p, _)| p.clone())
                .collect::<Vec<GridPoint>>();
            return Some((grid_point.clone(), grid_points));
        }
    }

    None
}

/// Return list of sorted by opacity points around
pub fn find_arbitrary_cover_grid_points<'a>(
    config: &'a ServerConfig,
    from_grid_point: &'a GridPoint,
    map: &'a Map,
    cover_distance: i32,
) -> Vec<(GridPoint, &'a TerrainTile)> {
    let mut tiles: Vec<(GridPoint, &TerrainTile)> = vec![];
    if let Some(tile) = map
        .terrain_tiles()
        .get((from_grid_point.y * map.width() as i32 + from_grid_point.x) as usize)
    {
        tiles.push((from_grid_point.clone(), tile))
    }
    let grid_points_for_square =
        grid_points_for_square(&from_grid_point, cover_distance, cover_distance);
    for grid_point in grid_points_for_square {
        if let Some(tile) = map
            .terrain_tiles()
            .get((grid_point.y * map.width() as i32 + grid_point.x) as usize)
        {
            tiles.push((grid_point, tile))
        }
    }
    tiles.sort_by(|(_, tile_a), (_, tile_b)| {
        config
            .terrain_tile_opacity(&tile_a.type_)
            .partial_cmp(&config.terrain_tile_opacity(&tile_b.type_))
            // TODO : manage this unwrap
            .unwrap()
    });

    tiles
}
