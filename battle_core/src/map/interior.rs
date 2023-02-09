#[derive(Clone)]
pub struct Interior {
    x: f32,
    relative_x: f32,
    y: f32,
    relative_y: f32,
    width: f32,
    relative_width: f32,
    height: f32,
    relative_height: f32,
}

impl Interior {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        image_width: f32,
        image_height: f32,
    ) -> Self {
        Self {
            x,
            relative_x: x / image_width,
            y,
            relative_y: y / image_height,
            width,
            relative_width: width / image_width,
            height,
            relative_height: height / image_height,
        }
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
