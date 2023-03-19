use ggez::graphics::DrawParam;

pub mod background;

pub trait Component {
    fn sprites(&self) -> Vec<DrawParam>;
}
