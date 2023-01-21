use crate::{
    behavior::{feeling::Feeling, gesture::Gesture, Behavior},
    config::{SOLDIER_SELECTABLE_SQUARE_SIDE, SOLDIER_SELECTABLE_SQUARE_SIDE_HALF},
    game::{
        weapon::{Magazine, Weapon},
        Side,
    },
    graphics::{animation::Sprite, soldier::SoldierAnimationType},
    order::Order,
    types::*,
};
use ggez::graphics::Rect;
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

    pub fn main_weapon_mut(&mut self) -> &mut Option<Weapon> {
        &mut self.main_weapon
    }

    pub fn magazines(&self) -> &Vec<Magazine> {
        &self.magazines
    }

    pub fn magazines_mut(&mut self) -> &mut Vec<Magazine> {
        &mut self.magazines
    }

    pub fn get_selection_rect(&self) -> Rect {
        Rect::new(
            self.world_point.x - SOLDIER_SELECTABLE_SQUARE_SIDE_HALF,
            self.world_point.y - SOLDIER_SELECTABLE_SQUARE_SIDE_HALF,
            SOLDIER_SELECTABLE_SQUARE_SIDE,
            SOLDIER_SELECTABLE_SQUARE_SIDE,
        )
    }

    pub fn get_animation_type(&self) -> Box<dyn Sprite> {
        let animation_type = match self.behavior() {
            Behavior::Idle => SoldierAnimationType::Idle,
            Behavior::MoveTo(_) => SoldierAnimationType::Walking,
            Behavior::MoveFastTo(_) => SoldierAnimationType::Walking,
            Behavior::SneakTo(_) => SoldierAnimationType::Crawling,
            Behavior::Defend(_) => SoldierAnimationType::LyingDown,
            Behavior::Hide(_) => SoldierAnimationType::LyingDown,
            Behavior::DriveTo(_) => SoldierAnimationType::Idle,
            Behavior::RotateTo(_) => SoldierAnimationType::Idle,
            // TODO : Different animation according to death type
            Behavior::Dead => SoldierAnimationType::DeadWithSideBlood,
            Behavior::Unconscious => SoldierAnimationType::LyingDown,
            Behavior::SuppressFire(_) => SoldierAnimationType::LyingDown,
            Behavior::EngageSoldier(_) => SoldierAnimationType::LyingDown,
        };
        Box::new(animation_type)
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

    pub fn increase_under_fire(&mut self, value: u32) {
        self.under_fire.increase(value)
    }

    pub fn decrease_under_fire(&mut self) {
        self.under_fire.decrease()
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
        if let Some(weapon) = self.weapon_mut(class) {
            weapon.reload();
        }
    }

    pub fn weapon_shot(&mut self, class: &WeaponClass) {
        if let Some(weapon) = self.weapon_mut(class) {
            weapon.shot();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WeaponClass {
    Main,
}
