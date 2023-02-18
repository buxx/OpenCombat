use battle_core::{
    behavior::Behavior,
    types::{Angle, WorldPoint},
    utils::angle,
};

use super::Runner;

impl Runner {
    pub fn behavior_angle(
        &self,
        behavior: &Behavior,
        reference_point: &WorldPoint,
    ) -> Option<Angle> {
        match behavior {
            Behavior::Idle => None,
            Behavior::MoveTo(paths) | Behavior::MoveFastTo(paths) | Behavior::SneakTo(paths) => {
                if let Some(next_point) = paths.next_point() {
                    Some(angle(&next_point, &reference_point))
                } else {
                    None
                }
            }
            Behavior::Defend(angle) => Some(*angle),
            Behavior::Hide(angle) => Some(*angle),
            Behavior::DriveTo(_) => None,
            Behavior::RotateTo(_) => None,
            Behavior::SuppressFire(point) => Some(angle(&point, &reference_point)),
            // FIXME BS NOW : don't know yet which soldier ?! :(
            Behavior::EngageSoldier(_) => None,
            // TODO: keep angle for dead/unconscious soldiers
            Behavior::Dead | Behavior::Unconscious => None,
        }
    }
}
