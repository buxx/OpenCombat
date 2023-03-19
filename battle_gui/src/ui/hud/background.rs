use ggez::graphics::DrawParam;

use crate::ui::component::Component;

pub struct Background {
    sprites: Vec<DrawParam>,
}

impl Background {
    pub fn new(sprites: Vec<DrawParam>) -> Self {
        Self { sprites }
    }
}

impl Component for Background {
    fn sprites(&self) -> Vec<DrawParam> {
        self.sprites.clone()
    }
}
