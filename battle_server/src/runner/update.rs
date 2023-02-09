use battle_core::{
    behavior::Behavior,
    state::battle::message::{BattleStateMessage, SoldierMessage},
    types::SoldierIndex,
};

use super::{message::RunnerMessage, Runner};

impl Runner {
    pub fn update_soldier(&self, i: SoldierIndex) -> Vec<RunnerMessage> {
        puffin::profile_scope!("update_soldier", format!("{}", i));
        let mut messages = vec![];

        messages.extend(self.orientation_update(i));
        messages.extend(self.behavior_update(i));

        messages
    }

    fn orientation_update(&self, i: SoldierIndex) -> Vec<RunnerMessage> {
        let soldier = self.battle_state.soldier(i);
        let mut messages = vec![];

        if let Some(angle_) = self.behavior_angle(soldier.behavior(), &soldier.get_world_point()) {
            let soldier_message = SoldierMessage::SetOrientation(angle_);
            messages.push(RunnerMessage::BattleState(BattleStateMessage::Soldier(
                i,
                soldier_message,
            )));
        }

        messages
    }

    fn behavior_update(&self, soldier_index: SoldierIndex) -> Vec<RunnerMessage> {
        let soldier = self.battle_state.soldier(soldier_index);
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
