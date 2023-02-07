use rayon::prelude::*;
use std::collections::HashMap;

use crate::{
    audio::Sound,
    entity::soldier::Soldier,
    game::Side,
    message::{LocalStateMessage, Message, SharedStateMessage, SoldierMessage},
    order::Order,
    physics::visibility::Visibility,
    types::{SoldierIndex, SquadUuid, WorldPoint},
};

use super::Engine;

impl Engine {
    pub fn tick_visibilities(&self) -> Vec<Message> {
        puffin::profile_scope!("tick_visibilities");
        let mut messages = vec![];
        let tick_visibility =
            self.local_state.get_frame_i() % self.config.visibility_update_freq() == 0;

        if tick_visibility {
            messages.extend(self.update_visibilities());
            messages.extend(self.update_orders_due_to_visibilities());
        }

        messages
    }

    pub fn update_visibilities(&self) -> Vec<Message> {
        let side_a_soldiers: Vec<&Soldier> = self
            .shared_state
            .soldiers()
            .iter()
            .filter(|s| s.get_side() == &Side::A)
            .collect();
        let side_b_soldiers: Vec<&Soldier> = self
            .shared_state
            .soldiers()
            .iter()
            .filter(|s| s.get_side() == &Side::B)
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

        vec![Message::LocalState(LocalStateMessage::SetVisibilities(
            from_side_a_visibilities
                .into_iter()
                .chain(from_side_b_visibilities)
                .collect(),
        ))]
    }

    pub fn soldier_visibilities(
        &self,
        soldier_index: SoldierIndex,
        other_soldiers: &Vec<&Soldier>,
    ) -> HashMap<(SoldierIndex, SoldierIndex), Visibility> {
        let mut visibilities = HashMap::new();
        let soldier = self.shared_state.soldier(soldier_index);

        if !soldier.can_seek() {
            return visibilities;
        }

        for other_soldier in other_soldiers {
            visibilities.insert(
                (soldier.uuid(), other_soldier.uuid()),
                Visibility::between_soldiers(
                    self.local_state.get_frame_i(),
                    &self.config,
                    soldier,
                    other_soldier,
                    &self.map,
                ),
            );
        }

        visibilities
    }

    pub fn soldier_is_visible_by_side(&self, soldier: &Soldier, side: &Side) -> bool {
        for visibility in self
            .local_state
            .visibilities()
            .visibles_soldiers()
            .iter()
            .filter(|v| self.shared_state.soldier(v.from_soldier).get_side() == side)
        {
            if visibility.to_soldier == Some(soldier.uuid()) {
                return true;
            }
        }

        false
    }

    pub fn point_is_visible_by_squad(&self, point: &WorldPoint, squad_index: &SquadUuid) -> bool {
        self.shared_state
            .squad(*squad_index)
            .members()
            .iter()
            .map(|i| self.shared_state.soldier(*i))
            .any(|s| {
                Visibility::between_soldier_and_point(
                    self.local_state.get_frame_i(),
                    &self.config,
                    s,
                    point,
                    &self.map,
                )
                .visible
            })
    }

    pub fn point_is_visible_by_soldier(&self, soldier: &Soldier, point: &WorldPoint) -> bool {
        Visibility::between_soldier_and_point(
            self.local_state.get_frame_i(),
            &self.config,
            soldier,
            point,
            &self.map,
        )
        .visible
    }

    fn update_orders_due_to_visibilities(&self) -> Vec<Message> {
        let mut messages = vec![];

        for side in [Side::A, Side::B] {
            for (squad_index, order) in self.shared_state.all_orders(&side) {
                match order {
                    Order::Idle
                    | Order::MoveTo(_)
                    | Order::MoveFastTo(_)
                    | Order::SneakTo(_)
                    | Order::Defend(_)
                    | Order::Hide(_)
                    | Order::SuppressFire(_) => {}
                    Order::EngageSquad(squad_uuid) => {
                        let engaged_squad = self.shared_state.squad(squad_uuid);
                        if !engaged_squad
                            .members()
                            .iter()
                            .map(|i| self.shared_state.soldier(*i))
                            .map(|s| self.soldier_is_visible_by_side(s, &side))
                            .any(|v| v)
                        {
                            messages.extend(vec![
                                Message::SharedState(SharedStateMessage::Soldier(
                                    self.shared_state.squad(squad_index).leader(),
                                    SoldierMessage::SetOrder(Order::Idle),
                                )),
                                Message::SharedState(SharedStateMessage::PushSoundToPlay(
                                    Sound::Bip1,
                                )),
                            ]);
                        }
                    }
                }
            }
        }

        messages
    }
}
