use crate::tile::WorldTileIndex;

pub mod region;
pub mod tile;
pub mod world;

pub trait Geo {
    fn tile(&self) -> WorldTileIndex;
}

pub trait UpdateGeo {
    fn set_tile(&mut self, value: WorldTileIndex);
}
