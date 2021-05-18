use crate::behavior::ItemBehavior;
use crate::config::{MOVE_FAST_VELOCITY, MOVE_HIDE_VELOCITY, MOVE_VELOCITY};

pub fn velocity_for_behavior(behavior: &ItemBehavior) -> Option<f32> {
    match behavior {
        ItemBehavior::MoveTo(_, _) => Some(MOVE_VELOCITY),
        ItemBehavior::MoveFastTo(_, _) => Some(MOVE_FAST_VELOCITY),
        ItemBehavior::HideTo(_, _) => Some(MOVE_HIDE_VELOCITY),
        _ => None,
    }
}
