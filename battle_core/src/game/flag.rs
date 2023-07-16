#[derive(Clone)]
pub struct Flag {
    name: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Flag {
    pub fn new(name: String, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            name,
            x,
            y,
            width,
            height,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
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
}
