use crate::{behavior::Behavior, message::*, types::*};

use super::Engine;

impl Engine {
    ///  - World pixel point according to movement
    ///  - ...
    pub fn update_soldier(&self, i: SoldierIndex) -> Vec<Message> {
        let mut messages = vec![];

        messages.extend(self.orientation_update(i));
        messages.extend(self.behavior_update(i));

        messages
    }

    fn orientation_update(&self, i: SoldierIndex) -> Vec<Message> {
        let soldier = self.shared_state.soldier(i);
        let mut messages = vec![];

        if let Some(angle_) = self.behavior_angle(soldier.behavior(), &soldier.get_world_point()) {
            let soldier_message = SoldierMessage::SetOrientation(angle_);
            messages.push(Message::SharedState(SharedStateMessage::Soldier(
                i,
                soldier_message,
            )));
        }

        messages
    }

    fn behavior_update(&self, soldier_index: SoldierIndex) -> Vec<Message> {
        let soldier = self.shared_state.soldier(soldier_index);
        let mut messages = vec![];

        messages.extend(match soldier.behavior() {
            Behavior::Idle => {
                vec![]
            }
            Behavior::MoveTo(paths) | Behavior::MoveFastTo(paths) | Behavior::SneakTo(paths) => {
                self.movement_updates(soldier_index, paths)
            }
            Behavior::Defend(_) => {
                vec![]
            }
            Behavior::Hide(_) => {
                vec![]
            }
            Behavior::DriveTo(paths) => self.drive_update(soldier_index, &paths),
            Behavior::RotateTo(_) => todo!(),
            Behavior::SuppressFire(_) => {
                vec![]
            }
            Behavior::EngageSoldier(_) => vec![],
            Behavior::Dead => vec![],
            Behavior::Unconscious => vec![],
        });

        messages
    }
}
