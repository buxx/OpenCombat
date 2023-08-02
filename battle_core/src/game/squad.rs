use std::collections::HashMap;

use oc_core::morale::Morale;

use crate::{
    entity::soldier::Soldier,
    state::battle::BattleState,
    types::{SoldierIndex, SquadComposition, WorldPoint},
    utils::apply_angle_on_point,
};

use super::{
    health::{Health, SoldierHealthBuilder},
    weapon::{Magazine, Weapon},
    Side,
};

pub enum Formation {
    Line,
}

pub fn squad_positions(
    squad: &SquadComposition,
    formation: Formation,
    leader: &Soldier,
    point: Option<WorldPoint>,
) -> HashMap<SoldierIndex, WorldPoint> {
    let mut positions = HashMap::new();
    let ref_point = point.unwrap_or(leader.world_point());
    let ref_angle = leader.get_looking_direction();

    match formation {
        Formation::Line => {
            let mut x_offset: f32 = 0.0;
            let mut y_offset: f32 = 0.0;
            let mut counter: u8 = 0;

            for (i, soldier_index) in squad.members().iter().enumerate() {
                // Don't return position for leader
                if *soldier_index == squad.leader() {
                    continue;
                }

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

                let member_scene_point =
                    WorldPoint::new(ref_point.x + x_offset_, ref_point.y + y_offset_);
                let member_scene_point =
                    apply_angle_on_point(&member_scene_point, &ref_point, &ref_angle);
                positions.insert(*soldier_index, member_scene_point);
            }
        }
    }

    positions
}

pub struct SquadStatusesResume {
    squads: Vec<SquadStatusResume>,
}

impl SquadStatusesResume {
    pub fn from_battle_state(side: &Side, battle_state: &BattleState) -> Self {
        Self {
            squads: battle_state
                .squads()
                .iter()
                .filter(|(_, squad)| battle_state.soldier(squad.leader()).side() == side)
                .map(|(_, squad)| SquadStatusResume::from_squad(battle_state, squad))
                .collect(),
        }
    }

    pub fn squads(&self) -> &[SquadStatusResume] {
        self.squads.as_ref()
    }
}

pub struct SquadStatusResume {
    health: SquadHealth,
    members: Vec<SquadMemberStatus>,
}

impl SquadStatusResume {
    pub fn from_squad(battle_state: &BattleState, squad: &SquadComposition) -> Self {
        Self {
            health: SquadHealth::from_squad(battle_state, squad),
            members: squad
                .members()
                .iter()
                .map(|soldier_index| {
                    SquadMemberStatus::from_soldier(
                        battle_state,
                        squad,
                        battle_state.soldier(*soldier_index),
                    )
                })
                .collect(),
        }
    }

    pub fn health(&self) -> &SquadHealth {
        &self.health
    }

    pub fn members(&self) -> &[SquadMemberStatus] {
        self.members.as_ref()
    }
}

pub struct SquadHealth(pub f32);

impl SquadHealth {
    pub fn from_squad(battle_state: &BattleState, squad: &SquadComposition) -> Self {
        let total = squad.members().len();
        let readies = squad
            .members()
            .iter()
            .filter(|soldier_index| {
                battle_state
                    .soldier(**soldier_index)
                    .can_be_count_for_morale()
            })
            .collect::<Vec<&SoldierIndex>>()
            .len();
        Self(readies as f32 / total as f32)
    }

    pub fn into_morale(&self) -> Morale {
        Morale(self.0)
    }
}

pub struct SquadMemberStatus {
    health: Health,
    main_weapon: Option<Weapon>,
    magazines: Vec<Magazine>,
}

impl SquadMemberStatus {
    pub fn from_soldier(
        _battle_state: &BattleState,
        _squad: &SquadComposition,
        soldier: &Soldier,
    ) -> Self {
        Self {
            health: SoldierHealthBuilder::new(soldier).build(),
            main_weapon: soldier.main_weapon().clone(),
            magazines: soldier.magazines().clone(),
        }
    }

    pub fn main_weapon(&self) -> Option<&Weapon> {
        self.main_weapon.as_ref()
    }

    pub fn health(&self) -> &Health {
        &self.health
    }

    pub fn magazines(&self) -> &[Magazine] {
        self.magazines.as_ref()
    }
}
