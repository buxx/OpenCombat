use rayon::prelude::*;
use std::collections::HashMap;

use battle_core::{
    audio::Sound,
    entity::soldier::Soldier,
    game::Side,
    order::Order,
    physics::visibility::Visibility,
    state::{
        battle::message::{BattleStateMessage, SoldierMessage},
        client::ClientStateMessage,
    },
    types::SoldierIndex,
};

use super::{message::RunnerMessage, Runner};

impl Runner {
    pub fn tick_visibilities(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_visibilities");
        let mut messages = vec![];

        if self.battle_state.soldiers().is_empty() {
            return vec![];
        }

        if self.is_tick_update_soldier_freq() {
            messages.extend(self.update_soldier_visibilities());
            messages.push(RunnerMessage::IncrementVisibilityIndex);
        }
        if self.is_tick_update_orders() {
            messages.extend(self.update_orders_due_to_visibilities());
        }

        messages
    }

    fn update_soldier_freq(&self) -> u64 {
        let soldiers_can_seeks = self.battle_state.soldiers();
        if soldiers_can_seeks.len() >= self.config.visibility_update_freq() as usize {
            1
        } else {
            self.config.visibility_update_freq() / soldiers_can_seeks.len() as u64
        }
    }

    fn is_tick_update_orders(&self) -> bool {
        self.battle_state.frame_i() % self.config.visibility_update_freq() == 0
            && self.battle_state.phase().is_battle()
    }

    fn is_tick_update_soldier_freq(&self) -> bool {
        self.battle_state.frame_i() % self.update_soldier_freq() == 0
            && self.battle_state.phase().is_battle()
    }

    pub fn update_soldier_visibilities(&self) -> Vec<RunnerMessage> {
        let soldier = self
            .battle_state
            .soldier(SoldierIndex(self.current_visibility));

        let other_soldiers: Vec<&Soldier> = self
            .battle_state
            .soldiers()
            .iter()
            .filter(|s| s.side() == &soldier.side().opposite())
            .collect();

        let visibilities: HashMap<(SoldierIndex, SoldierIndex), Visibility> = vec![soldier.uuid()]
            .into_par_iter()
            .flat_map(|i| self.soldier_visibilities(i, &other_soldiers))
            .collect();

        vec![RunnerMessage::BattleState(
            BattleStateMessage::SetVisibilities(visibilities),
        )]
    }

    pub fn soldier_visibilities(
        &self,
        soldier_index: SoldierIndex,
        other_soldiers: &Vec<&Soldier>,
    ) -> HashMap<(SoldierIndex, SoldierIndex), Visibility> {
        let mut visibilities = HashMap::new();
        let soldier = self.battle_state.soldier(soldier_index);

        if !soldier.can_seek() {
            for other_soldier in other_soldiers {
                visibilities.insert(
                    (soldier.uuid(), other_soldier.uuid()),
                    Visibility::between_soldiers_no(soldier, other_soldier),
                );
            }
        } else {
            for other_soldier in other_soldiers {
                visibilities.insert(
                    (soldier.uuid(), other_soldier.uuid()),
                    Visibility::between_soldiers(
                        *self.battle_state.frame_i(),
                        &self.config,
                        soldier,
                        other_soldier,
                        self.battle_state.map(),
                    ),
                );
            }
        }

        visibilities
    }

    fn update_orders_due_to_visibilities(&self) -> Vec<RunnerMessage> {
        let mut messages = vec![];

        for side in [Side::A, Side::B] {
            for (squad_index, order) in self.battle_state.all_orders(&side) {
                match order {
                    Order::Idle
                    | Order::MoveTo(_, _)
                    | Order::MoveFastTo(_, _)
                    | Order::SneakTo(_, _)
                    | Order::Defend(_)
                    | Order::Hide(_)
                    | Order::SuppressFire(_) => {}
                    Order::EngageSquad(squad_uuid) => {
                        let engaged_squad = self.battle_state.squad(*squad_uuid);
                        if !engaged_squad
                            .members()
                            .iter()
                            .map(|i| self.battle_state.soldier(*i))
                            .map(|s| self.battle_state.soldier_is_visible_by_side(s, &side))
                            .any(|v| v)
                        {
                            messages.extend(vec![
                                RunnerMessage::BattleState(BattleStateMessage::Soldier(
                                    self.battle_state.squad(squad_index).leader(),
                                    SoldierMessage::SetOrder(Order::Idle),
                                )),
                                RunnerMessage::ClientState(
                                    side,
                                    ClientStateMessage::PlayInterfaceSound(Sound::Bip1),
                                ),
                            ]);
                        }
                    }
                }
            }
        }

        messages
    }
}
