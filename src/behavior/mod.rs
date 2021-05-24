pub mod animate;
pub mod engagement;
pub mod movement;
pub mod order;
pub mod standing;

use crate::physics::GridPoint;
use crate::{GridPath, SceneItemId, ScenePoint};

#[derive(PartialEq)]
pub enum ItemBehavior {
    Standing, // since
    HideTo(ScenePoint, GridPath),
    MoveTo(ScenePoint, GridPath),
    MoveFastTo(ScenePoint, GridPath),
    EngageSceneItem(SceneItemId),
    EngageGridPoint(GridPoint),
}
