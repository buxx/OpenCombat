use crate::physics::util::apply_angle_on_point;
use crate::{Angle, SceneItemId, ScenePoint};
use std::collections::HashMap;

pub enum Formation {
    Line,
}

pub struct Squad {
    pub formation: Formation,
    pub leader: SceneItemId,
    pub members: Vec<SceneItemId>,
}

impl Squad {
    pub fn new() -> Self {
        Self {
            formation: Formation::Line,
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
            Formation::Line => {
                let member_ids = self
                    .members
                    .iter()
                    .filter(|id| **id != self.leader)
                    .collect::<Vec<&SceneItemId>>();
                let mut x_offset: f32 = 0.0;
                let mut y_offset: f32 = 0.0;
                let mut counter: u8 = 0;

                for (i, member_id) in member_ids.iter().enumerate() {
                    if counter % 2 == 0 {
                        x_offset += 10.0;
                        y_offset += 0.0;
                    }
                    counter += 1;

                    let (x_offset_, y_offset_) = if i % 2 == 0 {
                        (x_offset, y_offset)
                    } else {
                        (-x_offset, -y_offset)
                    };

                    let member_scene_point = ScenePoint::new(
                        leader_position.x + x_offset_,
                        leader_position.y + y_offset_,
                    );
                    let member_scene_point =
                        apply_angle_on_point(&member_scene_point, leader_position, &angle);
                    positions.insert(**member_id, member_scene_point);
                }
            }
        }

        positions
    }
}
