use crate::ScenePoint;

#[derive(Clone)]
pub enum Order {
    MoveTo(ScenePoint),
}
