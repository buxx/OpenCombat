use oc_individual::Individual;
use oc_root::WorldConfig;
use oc_world::tile::Tile;

pub trait IndividualsGenerator {
    fn individuals(&self, w: &WorldConfig, tiles: &[Tile]) -> Vec<Individual>;
}
