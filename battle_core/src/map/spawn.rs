use oc_core::spawn::SpawnZoneName;

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
}
