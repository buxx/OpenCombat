use battle_core::{
    state::battle::message::{BattleStateMessage, SoldierMessage},
    types::SoldierIndex,
};

use super::{message::RunnerMessage, Runner};

impl Runner {
    // TODO : Soldiers in vehicles must be managed differently than ground soldiers
    pub fn tick_soldiers(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_soldiers");
        let mut messages = vec![];
        let tick_animate = self.frame_i % self.config.soldier_animate_freq() == 0
            && self.battle_state.phase().is_battle();
        let tick_update = self.frame_i % self.config.soldier_update_freq() == 0;

        // Entities animation
        if tick_animate {
            messages.extend(
                (0..self.battle_state.soldiers().len())
                    // TODO : For now, parallel iter cost more than serial
                    // .into_par_iter()
                    .flat_map(|i| self.animate_soldier(SoldierIndex(i)))
                    .collect::<Vec<RunnerMessage>>(),
            );
        }

        // Entities updates
        if tick_update {
            let soldier_messages: Vec<RunnerMessage> = (0..self.battle_state.soldiers().len())
                // TODO : For now, parallel iter cost more than serial
                // .into_par_iter()
                .flat_map(|i| self.update_soldier(SoldierIndex(i)))
                .collect();
            messages.extend(soldier_messages);
        }

        messages
    }

    pub fn tick_feeling_decreasing_soldiers(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_feeling_decreasing_soldiers");
        let mut messages = vec![];
        let tick_feeling_decreasing = self.frame_i % self.config.feeling_decreasing_freq() == 0
            && self.battle_state.phase().is_battle();

        if tick_feeling_decreasing {
            messages.extend((0..self.battle_state.soldiers().len()).map(|i| {
                RunnerMessage::BattleState(BattleStateMessage::Soldier(
                    SoldierIndex(i),
                    SoldierMessage::DecreaseUnderFire,
                ))
            }));
        }

        messages
    }

    pub fn soldier_is_squad_leader(&self, soldier_index: SoldierIndex) -> bool {
        let soldier = self.battle_state.soldier(soldier_index);
        let squad_uuid = soldier.squad_uuid();
        let squad_composition = self.battle_state.squad(squad_uuid);
        let squad_leader = squad_composition.leader();
        squad_leader == soldier_index
    }

    pub fn animate_soldier(&self, soldier_index: SoldierIndex) -> Vec<RunnerMessage> {
        puffin::profile_scope!("animate_soldier", format!("{}", soldier_index));
        let soldier = self.battle_state.soldier(soldier_index);
        if !soldier.can_be_animated() {
            return vec![];
        }

        let mut messages = vec![];

        messages.extend(self.soldier_behavior(soldier));
        messages.extend(self.soldier_gesture(soldier));

        messages
    }

    pub fn tick_update_squad_leaders(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_update_squad_leaders");
        let mut messages = vec![];
        let tick_update = self.frame_i % self.config.squad_leaders_update_freq() == 0;

        if tick_update {
            for squad_uuid in self.battle_state.squads().keys() {
                let squad = self.battle_state.squad(*squad_uuid);
                let leader = self.battle_state.soldier(squad.leader());

                if !leader.can_be_leader() {
                    if let Some(member) = squad
                        .subordinates()
                        .iter()
                        .map(|s| self.battle_state.soldier(**s))
                        .find(|s| s.can_be_leader())
                    {
                        messages.push(RunnerMessage::BattleState(
                            BattleStateMessage::SetSquadLeader(*squad_uuid, member.uuid()),
                        ))
                    }
                }
            }
        }
        messages
    }
}
