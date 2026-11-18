use std::path::PathBuf;

use crate::{
    decor::Decor,
    flag::{Flag, FlagName},
    interior::Interior,
    place::Place,
    spawn::{SpawnZone, SpawnZoneName},
};

#[derive(Clone)]
pub struct Map {
    background_image_path: PathBuf,
    interiors_image_path: PathBuf,
    terrain_image_path: PathBuf,
    interiors: Vec<Interior>,
    spawn_zones: Vec<SpawnZone>,
    width: u32,
    height: u32,
    decor: Vec<Option<Decor>>,
    flags: Vec<Flag>,
    places: Vec<Place>,
}

impl Map {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        background_image_path: PathBuf,
        interiors_image_path: PathBuf,
        terrain_image_path: PathBuf,
        interiors: Vec<Interior>,
        spawn_zones: Vec<SpawnZone>,
        width: u32,
        height: u32,
        decor: Vec<Option<Decor>>,
        flags: Vec<Flag>,
        places: Vec<Place>,
    ) -> Self {
        Self {
            background_image_path,
            interiors_image_path,
            terrain_image_path,
            interiors,
            spawn_zones,
            width,
            height,
            decor,
            flags,
            places,
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

    pub fn decor(&self) -> &[Option<Decor>] {
        &self.decor
    }

    pub fn flags(&self) -> &Vec<Flag> {
        &self.flags
    }

    pub fn places(&self) -> &Vec<Place> {
        &self.places
    }

    // TODO : Get flags by name is not clean way, it could be better to use indexes (like soldiers, etc)
    pub fn flag(&self, flag_name: &FlagName) -> &Flag {
        self.flags()
            .iter()
            .find(|f| &f.name == flag_name)
            .expect("Flags ownership and map flag must be consistent")
    }

    pub fn find_spawn_zones(&self, names: &[SpawnZoneName]) -> Vec<&SpawnZone> {
        self.spawn_zones
            .iter()
            .filter(|s| names.contains(&s.name))
            .collect()
    }

    // pub fn spawn_contains_flag(&self, spawns: &[SpawnZone], flag: &Flag) -> bool {
    //     for spawn_zone_name in spawns {
    //         // FIXME: algo moche ?!
    //         let found = self.find_spawn_zones(std::slice::from_ref(spawn_zone_name));
    //         let spawn_zone = found.first().unwrap(); // FIXME: manage error

    //         if spawn_zone.include_shape(&flag.shape()) {
    //             return true;
    //         }
    //     }

    //     false
    // }

    // pub fn successors(
    //     &self,
    //     from: &(GridPoint, Direction),
    //     path_mode: &PathMode,
    // ) -> Vec<((GridPoint, Direction), i32)> {
    //     let mut successors = vec![];

    //     for direction in Direction::iter() {
    //         let (mod_x, mod_y) = direction.modifier();
    //         let new_x = from.0.x + mod_x;
    //         let new_y = from.0.y + mod_y;

    //         // Don't care ifd outside map
    //         if new_x < 0 || new_y < 0 || new_x > self.width as i32 || new_y > self.height as i32 {
    //             continue;
    //         }

    //         // If in map
    //         let i = (new_y * self.width() as i32 + new_x) as usize;
    //         if let Some(next_tile) = self.terrain_tiles.get(i) {
    //             if path_mode.include_vehicles() {
    //                 if next_tile.type_().block_vehicle() {
    //                     continue;
    //                 }

    //                 if let PathMode::Drive(size) = path_mode {
    //                     if !self.point_allow_vehicle(&GridPoint::new(new_x, new_y), size) {
    //                         continue;
    //                     }
    //                 }
    //             }

    //             let cost = match path_mode {
    //                 PathMode::Walk => next_tile.type_().pedestrian_cost(),
    //                 PathMode::Drive(_size) => from.1.angle_cost(&direction),
    //             };

    //             successors.push(((GridPoint::new(new_x, new_y), direction), cost))
    //         }
    //     }

    //     successors
    // }

    // pub fn contains(&self, grid_point: &GridPoint) -> bool {
    //     grid_point.x >= 0
    //         && grid_point.y >= 0
    //         && grid_point.x < self.width() as i32
    //         && grid_point.y < self.height() as i32
    // }

    // pub fn grid_point_from_world_point(&self, world_point: &WorldPoint) -> GridPoint {
    //     let x = world_point.x as u32 / self.tile_width();
    //     let y = world_point.y as u32 / self.tile_height();
    //     GridPoint::new(x as i32, y as i32)
    // }

    // pub fn world_point_from_grid_point(&self, grid_point: GridPoint) -> WorldPoint {
    //     let x = (grid_point.x * self.tile_width() as i32) + (self.tile_width() as i32 / 2);
    //     let y = (grid_point.y * self.tile_height() as i32) + (self.tile_height() as i32 / 2);
    //     WorldPoint::new(x as f32, y as f32)
    // }

    // pub fn point_allow_vehicle(&self, point: &GridPoint, size: &VehicleSize) -> bool {
    //     let half = (size.0 / 2) as i32;
    //     let start_x = point.x - half;
    //     let end_x = point.x + half;
    //     let start_y = point.y - half;
    //     let end_y = point.y + half;

    //     for x in start_x..end_x {
    //         for y in start_y..end_y {
    //             if let Some(tile) = self
    //                 .terrain_tiles
    //                 .get((y * self.width() as i32 + x) as usize)
    //             {
    //                 if tile.type_().block_vehicle() {
    //                     return false;
    //                 }
    //             }
    //         }
    //     }

    //     true
    // }
    //
    // pub fn point_in_spawn_zones(&self, point: &WorldVec2, allowed_zone_names: &MapControl) -> bool {
    //     for spawn_zone in &self.spawn_zones {
    //         if allowed_zone_names.contains_spawn_zone(spawn_zone.name())
    //             && point.x >= spawn_zone.x()
    //             && point.x <= spawn_zone.x() + spawn_zone.width()
    //             && point.y >= spawn_zone.y()
    //             && point.y <= spawn_zone.y() + spawn_zone.height()
    //         {
    //             return true;
    //         }
    //     }

    //     false
    // }
}
