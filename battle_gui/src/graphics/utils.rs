use ggez::graphics::DrawParam;

pub trait IntoDrawParam {
    fn to_draw_param(&self) -> DrawParam;
}
