pub mod animate;
pub mod defend;
pub mod engagement;
pub mod movement;
pub mod order;
pub mod standing;
pub mod util;

use crate::gameplay::weapon::SceneItemWeapon;
use crate::physics::GridPoint;
use crate::{GridPath, SceneItemId, ScenePoint};
use std::fmt;

pub enum ItemBehavior {
    Dead,
    Unconscious,
    Standing,
    HideTo(ScenePoint, GridPath),
    MoveTo(ScenePoint, GridPath),
    MoveFastTo(ScenePoint, GridPath),
    EngageSceneItem(SceneItemWeapon, SceneItemId),
    EngageGridPoint(GridPoint),
    Hide,
}

impl fmt::Display for ItemBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            ItemBehavior::Dead => "Dead",
            ItemBehavior::Unconscious => "Unconscious",
            ItemBehavior::Standing => "Standing",
            ItemBehavior::HideTo(_, _) => "HideTo",
            ItemBehavior::MoveTo(_, _) => "MoveTo",
            ItemBehavior::MoveFastTo(_, _) => "MoveFastTo",
            ItemBehavior::EngageSceneItem(_, _) => "EngageSceneItem",
            ItemBehavior::EngageGridPoint(_) => "EngageGridPoint",
            ItemBehavior::Hide => "Hide",
        };

        write!(f, "{}", string)
    }
}
