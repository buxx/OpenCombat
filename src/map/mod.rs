use std::path::PathBuf;

use self::{decor::Decor, interior::Interior, terrain::TerrainTile};
use crate::{
    config::{Config, COVER_DISTANCE},
    physics::path::{Direction, PathMode},
    types::{GridPoint, VehicleSize, WorldPoint},
    utils::grid_points_for_square,
};
use strum::IntoEnumIterator;

pub mod decor;
pub mod interior;
pub mod reader;
pub mod terrain;

pub struct Map {
    _name: String,
    background_image_path: PathBuf,
    interiors_image_path: PathBuf,
    terrain_image_path: PathBuf,
    interiors: Vec<Interior>,
    width: u32,
    height: u32,
    terrain_tiles: Vec<TerrainTile>,
    tile_width: u32,
    tile_height: u32,
    decor: Decor,
}

impl Map {
    pub fn new(
        name: String,
        background_image_path: PathBuf,
        interiors_image_path: PathBuf,
        terrain_image_path: PathBuf,
        interiors: Vec<Interior>,
        width: u32,
        height: u32,
        terrain_tiles: Vec<TerrainTile>,
        tile_width: u32,
        tile_height: u32,
        decor: Decor,
    ) -> Self {
        Self {
            _name: name,
            background_image_path,
            interiors_image_path,
            terrain_image_path,
            interiors,
            width,
            height,
            terrain_tiles,
            tile_width,
            tile_height,
            decor,
        }
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
                    if next_tile.block_vehicle {
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
                    PathMode::Walk => next_tile.pedestrian_cost,
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

    fn point_allow_vehicle(&self, point: &GridPoint, size: &VehicleSize) -> bool {
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
                    if tile.block_vehicle {
                        return false;
                    }
                }
            }
        }

        return true;
    }
}

pub fn find_cover_grid_point(
    config: &Config,
    from_grid_point: &GridPoint,
    map: &Map,
    exclude_grid_points: &Vec<GridPoint>,
) -> Option<(GridPoint, Vec<GridPoint>)> {
    let mut tiles: Vec<(GridPoint, &TerrainTile)> = vec![];
    if let Some(tile) = map
        .terrain_tiles()
        .get((from_grid_point.y * map.width() as i32 + from_grid_point.x) as usize)
    {
        tiles.push((from_grid_point.clone(), tile))
    }
    let grid_points_for_square =
        grid_points_for_square(&from_grid_point, COVER_DISTANCE, COVER_DISTANCE);
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
            .unwrap()
    });

    for (grid_point, _) in tiles.iter().rev() {
        if !exclude_grid_points.contains(grid_point) {
            let grid_points = tiles
                .iter()
                .map(|(p, _)| p.clone())
                .collect::<Vec<GridPoint>>();
            return Some((grid_point.clone(), grid_points));
        }
    }

    None
}
