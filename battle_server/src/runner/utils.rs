use battle_core::{
    behavior::Behavior,
    game::Side,
    physics::utils::meters_between_world_points,
    types::{Angle, Distance, WorldPoint},
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
            Behavior::Idle(_) => None,
            Behavior::MoveTo(paths) | Behavior::MoveFastTo(paths) | Behavior::SneakTo(paths) => {
                paths
                    .next_point()
                    .map(|next_point| angle(&next_point, reference_point))
            }
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

    // FIXME BS NOW : use this too for soldiers gestures : for now, we check from leader,
    // but soldier must defend himself too if is away
    pub fn visible_soldier_in_circle(
        &self,
        point: &WorldPoint,
        radius: &Distance,
        side: &Side,
    ) -> bool {
        self.battle_state
            .visibilities()
            .visibles_soldiers()
            .iter()
            .any(|visibility| {
                if let Some(soldier_index) = visibility.to_soldier {
                    let soldier = self.battle_state.soldier(soldier_index);
                    if soldier.side() == side
                        && &meters_between_world_points(point, &soldier.world_point()) <= radius
                    {
                        return true;
                    }
                }
                false
            })
    }
}
