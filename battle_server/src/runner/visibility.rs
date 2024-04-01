use rayon::prelude::*;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use battle_core::{
    audio::Sound,
    config::ServerConfig,
    entity::soldier::Soldier,
    game::Side,
    order::Order,
    physics::visibility::Visibility,
    state::{
        battle::{
            message::{BattleStateMessage, SoldierMessage},
            BattleState,
        },
        client::ClientStateMessage,
    },
    types::SoldierIndex,
};

use super::message::RunnerMessage;

pub struct VisibilityRunner {
    config: ServerConfig,
    battle_state: Arc<RwLock<BattleState>>,
}

impl VisibilityRunner {
    pub fn new(config: ServerConfig, battle_state: Arc<RwLock<BattleState>>) -> Self {
        Self {
            config,
            battle_state,
        }
    }

    pub fn battle_state(&self) -> RwLockReadGuard<'_, BattleState> {
        // FIXME unwrap
        self.battle_state.read().unwrap()
    }

    pub fn tick_visibilities(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_visibilities");
        let mut messages = vec![];
        let tick_visibility = self.battle_state().frame_i() % self.config.visibility_update_freq()
            == 0
            && self.battle_state().phase().is_battle();

        if tick_visibility {
            messages.extend(self.update_visibilities());
            messages.extend(self.update_orders_due_to_visibilities());
        }

        messages
    }

    pub fn update_visibilities(&self) -> Vec<RunnerMessage> {
        let side_a_soldiers: Vec<&Soldier> = self
            .battle_state()
            .soldiers()
            .iter()
            .filter(|s| s.side() == &Side::A)
            .collect();
        let side_b_soldiers: Vec<&Soldier> = self
            .battle_state()
            .soldiers()
            .iter()
            .filter(|s| s.side() == &Side::B)
            .collect();

        let from_side_a_visibilities: HashMap<(SoldierIndex, SoldierIndex), Visibility> =
            side_a_soldiers
                .iter()
                .map(|s| s.uuid())
                .collect::<Vec<SoldierIndex>>()
                .into_par_iter()
                .flat_map(|i| self.soldier_visibilities(i, &side_b_soldiers))
                .collect();
        let from_side_b_visibilities: HashMap<(SoldierIndex, SoldierIndex), Visibility> =
            side_b_soldiers
                .iter()
                .map(|s| s.uuid())
                .collect::<Vec<SoldierIndex>>()
                .into_par_iter()
                .flat_map(|i| self.soldier_visibilities(i, &side_a_soldiers))
                .collect();

        let visibilities = from_side_a_visibilities
            .into_iter()
            .chain(from_side_b_visibilities)
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
        let soldier = self.battle_state().soldier(soldier_index);

        if !soldier.can_seek() {
            return visibilities;
        }

        for other_soldier in other_soldiers {
            visibilities.insert(
                (soldier.uuid(), other_soldier.uuid()),
                Visibility::between_soldiers(
                    *self.battle_state().frame_i(),
                    &self.config,
                    soldier,
                    other_soldier,
                    self.battle_state().map(),
                ),
            );
        }

        visibilities
    }

    fn update_orders_due_to_visibilities(&self) -> Vec<RunnerMessage> {
        let mut messages = vec![];

        for side in [Side::A, Side::B] {
            for (squad_index, order) in self.battle_state().all_orders(&side) {
                match order {
                    Order::Idle
                    | Order::MoveTo(_, _)
                    | Order::MoveFastTo(_, _)
                    | Order::SneakTo(_, _)
                    | Order::Defend(_)
                    | Order::Hide(_)
                    | Order::SuppressFire(_) => {}
                    Order::EngageSquad(squad_uuid) => {
                        let engaged_squad = self.battle_state().squad(*squad_uuid);
                        if !engaged_squad
                            .members()
                            .iter()
                            .map(|i| self.battle_state().soldier(*i))
                            .map(|s| self.battle_state().soldier_is_visible_by_side(s, &side))
                            .any(|v| v)
                        {
                            messages.extend(vec![
                                RunnerMessage::BattleState(BattleStateMessage::Soldier(
                                    self.battle_state().squad(squad_index).leader(),
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
