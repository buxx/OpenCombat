use bresenham::Bresenham;
use rand::Rng;

use crate::{
    config::{COVERAGE_PIXEL_STEPS, COVERAGE_TILE_STEPS},
    entity::soldier::Soldier,
    map::Map,
    types::WorldPoint,
};

use super::event::bullet::BulletFire;

pub struct SoldierCovered<'a> {
    map: &'a Map,
    bullet_fire: &'a BulletFire,
    soldier: &'a Soldier,
}

impl<'a> SoldierCovered<'a> {
    pub fn new(map: &'a Map, bullet_fire: &'a BulletFire, soldier: &'a Soldier) -> Self {
        Self {
            map,
            bullet_fire,
            soldier,
        }
    }

    pub fn compute(&self, force_target_tile: bool) -> bool {
        // Make bullet path from the end to get target soldier tiles
        let pixels = Bresenham::new(
            (
                self.bullet_fire.to().x as isize,
                self.bullet_fire.to().y as isize,
            ),
            (
                self.bullet_fire.from().x as isize,
                self.bullet_fire.from().y as isize,
            ),
        );

        if force_target_tile {
            let target_grid_point = self
                .map
                .grid_point_from_world_point(&self.soldier.world_point());

            if let Some(tile) = self
                .map
                .terrain_tiles()
                .get((target_grid_point.y * self.map.width() as i32 + target_grid_point.x) as usize)
            {
                if let Some(coverage) = tile.type_().coverage(&self.soldier.behavior().posture()) {
                    let mut rng = rand::thread_rng();
                    let value: f32 = rng.gen();
                    return value <= coverage.0;
                }
            }
        }

        let mut visited_grid_points = vec![];
        for (pixel_x, pixel_y) in pixels.step_by(COVERAGE_PIXEL_STEPS) {
            let grid_point = self
                .map
                .grid_point_from_world_point(&WorldPoint::new(pixel_x as f32, pixel_y as f32));
            if let Some(tile) = self
                .map
                .terrain_tiles()
                .get((grid_point.y * self.map.width() as i32 + grid_point.x) as usize)
            {
                if let Some(coverage) = tile.type_().coverage(&self.soldier.behavior().posture()) {
                    let mut rng = rand::thread_rng();
                    let value: f32 = rng.gen();
                    return value <= coverage.0;
                }
            }

            if visited_grid_points.len() >= COVERAGE_TILE_STEPS {
                return false;
            }
            visited_grid_points.push(grid_point);
        }

        false
    }
}
