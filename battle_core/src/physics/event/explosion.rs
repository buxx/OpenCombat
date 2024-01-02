use crate::{config::TARGET_FPS, game::explosive::ExplosiveType, types::WorldPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explosion {
    start: u64,
    end: u64,
    point: WorldPoint,
    explosive_type: ExplosiveType,
}

impl Explosion {
    pub fn new(point: WorldPoint, type_: ExplosiveType) -> Self {
        Self {
            start: 0,
            end: 0,
            point,
            explosive_type: type_,
        }
    }

    pub fn init(&mut self, start_frame_i: u64) {
        self.start = start_frame_i;
        // FIXME BS NOW :  as u64 non non !
        self.end = start_frame_i + (self.explosive_type.sprite().duration() as u64 * TARGET_FPS);
    }

    pub fn point(&self) -> &WorldPoint {
        &self.point
    }

    pub fn type_(&self) -> &ExplosiveType {
        &self.explosive_type
    }

    pub fn finished(&self, frame_i: u64) -> bool {
        frame_i >= self.end
    }

    pub fn effective(&self, frame_i: u64) -> bool {
        self.start == frame_i
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        self.end
    }

    pub fn explosive_type(&self) -> &ExplosiveType {
        &self.explosive_type
    }
}
