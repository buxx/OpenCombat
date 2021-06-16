use crate::{Angle, SceneItemId, ScenePoint};
use std::collections::HashMap;

pub enum Formation {
    CenteredLine,
}

pub struct Squad {
    pub formation: Formation,
    pub leader: SceneItemId,
    pub members: Vec<SceneItemId>,
}

impl Squad {
    pub fn new() -> Self {
        Self {
            formation: Formation::CenteredLine,
            leader: 0,
            members: vec![],
        }
    }

    pub fn member_positions(
        &self,
        leader_position: &ScenePoint,
        angle: Angle,
    ) -> HashMap<SceneItemId, ScenePoint> {
        let mut positions = HashMap::new();
        match &self.formation {
            Formation::CenteredLine => {
                // FIXME BS NOW: hardcoded solution for test
                // FIXME BS NOW: take care from outside map !
                for (i, member_id) in self.members.iter().enumerate() {
                    let member_scene_point =
                        ScenePoint::new(leader_position.x + (10.0 * i as f32), leader_position.y);
                    let sin = f32::sin(angle);
                    let cos = f32::cos(angle);
                    let pt = (
                        member_scene_point.x - leader_position.x,
                        member_scene_point.y - leader_position.y,
                    );
                    let rotated = (
                        leader_position.x + pt.0 * cos - pt.1 * sin,
                        leader_position.y + pt.0 * sin + pt.1 * cos,
                    );
                    positions.insert(*member_id, ScenePoint::new(rotated.0, rotated.1));
                }
            }
        }

        positions
    }
}
