use oc_projectile::Projectile;
use oc_root::WorldConfig;
use oc_world::tile::Tile;

pub trait ProjectilesGenerator {
    fn projectiles(&self, w: &WorldConfig, tiles: &[Tile]) -> Vec<Projectile>;
}
