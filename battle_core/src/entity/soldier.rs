use crate::{
    behavior::{feeling::Feeling, gesture::Gesture, Behavior},
    deployment::SoldierDeployment,
    game::{
        weapon::{Magazine, Weapon},
        Side,
    },
    order::Order,
    types::*,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Soldier {
    uuid: SoldierIndex,
    side: Side,
    world_point: WorldPoint,
    squad_uuid: SquadUuid,
    order: Order,
    behavior: Behavior,
    gesture: Gesture,
    looking_direction: Angle,
    alive: bool,
    unconscious: bool,
    under_fire: Feeling,
    main_weapon: Option<Weapon>,
    magazines: Vec<Magazine>,
    last_shoot_frame_i: u64,
}

impl Soldier {
    pub fn new(
        uuid: SoldierIndex,
        world_point: WorldPoint,
        squad_uuid: SquadUuid,
        side: Side,
        main_weapon: Option<Weapon>,
        magazines: Vec<Magazine>,
    ) -> Self {
        Self {
            uuid,
            side,
            world_point,
            squad_uuid,
            order: Order::Idle,
            behavior: Behavior::Idle,
            gesture: Gesture::Idle,
            looking_direction: Angle(0.0),
            alive: true,
            unconscious: false,
            under_fire: Feeling::UnderFire(0),
            main_weapon,
            magazines,
            last_shoot_frame_i: 0,
        }
    }

    pub fn from_soldier(soldier: &Soldier) -> Self {
        Self::new(
            soldier.uuid(),
            soldier.get_world_point(),
            soldier.squad_uuid(),
            *soldier.get_side(),
            soldier.main_weapon().clone(),
            soldier.magazines().clone(),
        )
    }

    pub fn uuid(&self) -> SoldierIndex {
        self.uuid
    }

    pub fn get_side(&self) -> &Side {
        &self.side
    }

    pub fn get_world_point(&self) -> WorldPoint {
        self.world_point
    }

    pub fn set_world_point(&mut self, point: WorldPoint) {
        self.world_point = point
    }

    pub fn squad_uuid(&self) -> SquadUuid {
        self.squad_uuid
    }

    pub fn behavior(&self) -> &Behavior {
        &self.behavior
    }

    pub fn get_behavior_mut(&mut self) -> &mut Behavior {
        &mut self.behavior
    }

    pub fn gesture(&self) -> &Gesture {
        &self.gesture
    }

    pub fn set_gesture(&mut self, gesture: Gesture) {
        self.gesture = gesture
    }

    pub fn order(&self) -> &Order {
        &self.order
    }

    pub fn order_mut(&mut self) -> &mut Order {
        &mut self.order
    }

    pub fn set_behavior(&mut self, behavior: Behavior) {
        self.behavior = behavior
    }

    pub fn set_order(&mut self, order: Order) {
        println!("Soldier({})::set_order({})", self.uuid, &order);
        self.order = order
    }

    pub fn get_looking_direction(&self) -> Angle {
        self.looking_direction
    }

    pub fn set_looking_direction(&mut self, angle: Angle) {
        self.looking_direction = angle
    }

    pub fn main_weapon(&self) -> &Option<Weapon> {
        &self.main_weapon
    }

    pub fn magazines(&self) -> &Vec<Magazine> {
        &self.magazines
    }

    pub fn alive_mut(&mut self) -> &mut bool {
        &mut self.alive
    }

    pub fn unconscious_mut(&mut self) -> &mut bool {
        &mut self.unconscious
    }

    pub fn set_alive(&mut self, value: bool) {
        self.alive = value
    }

    pub fn set_unconscious(&mut self, value: bool) {
        self.unconscious = value
    }

    pub fn can_be_animated(&self) -> bool {
        self.alive && !self.unconscious
    }

    pub fn can_produce_sound(&self) -> bool {
        self.alive && !self.unconscious
    }

    pub fn can_feel_explosion(&self) -> bool {
        self.alive
    }

    pub fn can_feel_bullet_fire(&self) -> bool {
        self.alive
    }

    pub fn can_see_interior(&self) -> bool {
        self.alive && !self.unconscious
    }

    pub fn can_seek(&self) -> bool {
        self.alive && !self.unconscious
    }

    pub fn can_be_designed_as_target(&self) -> bool {
        self.alive && !self.unconscious
    }

    pub fn under_fire(&self) -> &Feeling {
        &self.under_fire
    }

    pub fn under_fire_mut(&mut self) -> &mut Feeling {
        &mut self.under_fire
    }

    pub fn increase_under_fire(&mut self, value: u32) {
        self.under_fire.increase(value)
    }

    pub fn decrease_under_fire(&mut self) {
        self.under_fire.decrease()
    }

    pub fn set_last_shoot_frame_i(&mut self, value: u64) {
        self.last_shoot_frame_i = value
    }

    pub fn last_shoot_frame_i(&self) -> &u64 {
        &self.last_shoot_frame_i
    }

    pub fn weapon(&self, class: &WeaponClass) -> &Option<Weapon> {
        match class {
            WeaponClass::Main => &self.main_weapon,
        }
    }

    pub fn weapon_mut(&mut self, class: &WeaponClass) -> &mut Option<Weapon> {
        match class {
            WeaponClass::Main => &mut self.main_weapon,
        }
    }

    pub fn reload_weapon(&mut self, class: &WeaponClass) {
        let mut magazines = self.magazines.clone();
        if let Some(weapon) = self.weapon_mut(class) {
            weapon.reload();
            if weapon.magazine().is_none() {
                while let Some(magazine) = magazines.pop() {
                    if weapon.accepted_magazine(&magazine) {
                        weapon.set_magazine(magazine);
                        break;
                    }
                }
            }
        }
        self.magazines = magazines;
    }

    pub fn weapon_shot(&mut self, class: &WeaponClass) {
        if let Some(weapon) = self.weapon_mut(class) {
            weapon.shot();
        }
    }

    pub fn alive(&self) -> bool {
        self.alive
    }

    pub fn unconscious(&self) -> bool {
        self.unconscious
    }

    pub fn target(&self) -> Option<&SoldierIndex> {
        match self.behavior() {
            Behavior::EngageSoldier(soldier_index) => Some(soldier_index),
            _ => None,
        }
    }
}

impl From<&SoldierDeployment> for Soldier {
    fn from(soldier: &SoldierDeployment) -> Self {
        Self::new(
            soldier.uuid().clone(),
            soldier.world_point().clone(),
            soldier.squad_uuid().clone(),
            soldier.side().clone(),
            soldier.main_weapon().cloned(),
            soldier.magazines().clone().to_vec(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WeaponClass {
    Main,
}
