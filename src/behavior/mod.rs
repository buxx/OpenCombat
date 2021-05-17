pub mod animate;
pub mod order;

use crate::ScenePoint;

#[derive(PartialEq)]
pub enum ItemBehavior {
    Standing, // since
    HideTo(ScenePoint),
    MoveTo(ScenePoint),
    MoveFastTo(ScenePoint),
}
