use battle_core::map::Map;

use super::MapModel;

pub struct MapGenerator<T: MapModel> {
    model: T,
    width: f32,
    height: f32,
}

impl<T: MapModel> MapGenerator<T> {
    pub fn new(model: T) -> Self {
        Self {
            model,
            width: Default::default(),
            height: Default::default(),
        }
    }

    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }

    pub fn height(mut self, value: f32) -> Self {
        self.height = value;
        self
    }

    pub fn generate(&self) -> Map {
        todo!()
    }
}
