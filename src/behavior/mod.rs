pub mod animate;
pub mod order;

use crate::{GridPath, ScenePoint};

#[derive(PartialEq)]
pub enum ItemBehavior {
    Standing, // since
    HideTo(ScenePoint, GridPath),
    MoveTo(ScenePoint, GridPath),
    MoveFastTo(ScenePoint, GridPath),
}
