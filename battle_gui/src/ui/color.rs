use ggez::graphics::Color;

pub trait Colorized {
    fn color(&self) -> Color;
}
