use ggez::graphics::DrawParam;

use super::qualified::Zoom;

pub trait Batch {
    fn clear(&mut self);
    fn push(&mut self, draw: DrawParam);

    fn extend(&mut self, draws: Vec<DrawParam>) {
        for param in draws {
            self.push(param)
        }
    }
}

pub trait QualifiedBatch<T> {
    fn hd(&self) -> &T;
    fn sd(&self) -> &T;

    fn clear(&mut self, zoom: &Zoom);
    fn push(&mut self, zoom: &Zoom, draw: DrawParam);

    fn extend(&mut self, zoom: &Zoom, draws: Vec<DrawParam>) {
        for param in draws {
            self.push(zoom, param)
        }
    }

    fn drawable(&self, zoom: &Zoom) -> &T {
        match zoom {
            Zoom::In => self.hd(),
            _ => self.sd(),
        }
    }
}
