use oc_core::spawn::SpawnZoneName;

use crate::{types::WorldPoint, utils::WorldShape};

#[derive(Clone)]
pub struct SpawnZone {
    name: SpawnZoneName,
    x: f32,
    relative_x: f32,
    y: f32,
    relative_y: f32,
    width: f32,
    relative_width: f32,
    height: f32,
    relative_height: f32,
}

impl SpawnZone {
    pub fn new(
        name: SpawnZoneName,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        map_width: f32,
        map_height: f32,
    ) -> Self {
        Self {
            name,
            x,
            relative_x: x / map_width,
            y,
            relative_y: y / map_height,
            width,
            relative_width: width / map_width,
            height,
            relative_height: height / map_height,
        }
    }

    pub fn name(&self) -> &SpawnZoneName {
        &self.name
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn relative_x(&self) -> f32 {
        self.relative_x
    }

    pub fn relative_y(&self) -> f32 {
        self.relative_y
    }

    pub fn relative_width(&self) -> f32 {
        self.relative_width
    }

    pub fn relative_height(&self) -> f32 {
        self.relative_height
    }

    pub fn shape(&self) -> WorldShape {
        WorldShape {
            top_left: WorldPoint::new(self.x, self.y),
            top_right: WorldPoint::new(self.x + self.width, self.y),
            bottom_right: WorldPoint::new(self.x + self.width, self.y + self.height),
            bottom_left: WorldPoint::new(self.x, self.y + self.height),
        }
    }

    pub fn contains(&self, shape: &WorldShape) -> bool {
        let this = self.shape();
        this.top_left.x <= shape.top_left.x
            && this.top_left.y <= shape.top_left.y
            && this.bottom_right.x >= shape.bottom_right.x
            && this.bottom_right.y >= shape.bottom_right.y
    }
}
