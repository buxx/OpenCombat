use battle_core::{
    behavior::Behavior,
    types::{Angle, WorldPoint},
    utils::angle,
};

pub fn behavior_angle(behavior: &Behavior, reference_point: &WorldPoint) -> Option<Angle> {
    match behavior {
        Behavior::Idle(_) => None,
        Behavior::MoveTo(paths) | Behavior::MoveFastTo(paths) | Behavior::SneakTo(paths) => paths
            .next_point()
            .map(|next_point| angle(&next_point, reference_point)),
        Behavior::Defend(angle) => Some(*angle),
        Behavior::Hide(angle) => Some(*angle),
        Behavior::DriveTo(_) => None,
        Behavior::RotateTo(_) => None,
        Behavior::SuppressFire(point) => Some(angle(point, reference_point)),
        Behavior::EngageSoldier(_) => None,
        // TODO: keep angle for dead/unconscious soldiers
        Behavior::Dead | Behavior::Unconscious => None,
    }
}
