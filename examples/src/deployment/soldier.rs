use battle_core::{
    behavior::{Behavior, Body},
    deployment::SoldierDeployment,
    game::{
        weapon::{Magazine, Weapon},
        Side,
    },
    order::Order,
    types::{SoldierIndex, SquadUuid, WorldPoint},
};
use oc_core::game::soldier::SoldierType;

pub struct ManualSoldiersGenerator {
    soldiers: Vec<SoldierDeployment>,
    type_: SoldierType,
    side: Side,
    squad: SquadUuid,
    main_weapon: Option<Weapon>,
    magazines: Vec<Magazine>,
    world_point: WorldPoint,
}

impl ManualSoldiersGenerator {
    pub fn type_(mut self, value: SoldierType) -> Self {
        self.type_ = value;
        self
    }

    pub fn side(mut self, value: Side) -> Self {
        self.side = value;
        self
    }

    pub fn squad(mut self, value: SquadUuid) -> Self {
        self.squad = value;
        self
    }

    pub fn main_weapon(mut self, value: Option<Weapon>) -> Self {
        self.main_weapon = value;
        self
    }

    pub fn magazines(mut self, value: Vec<Magazine>) -> Self {
        self.magazines = value;
        self
    }

    pub fn world_point(mut self, value: WorldPoint) -> Self {
        self.world_point = value;
        self
    }

    pub fn place<F>(mut self, count: usize, placer: F) -> Self
    where
        F: FnOnce(WorldPoint) -> WorldPoint + Copy,
    {
        for _ in 0..count {
            self.world_point = placer(self.world_point);
            let soldier = SoldierDeployment::new(
                SoldierIndex(self.soldiers.len()),
                self.type_,
                self.side,
                self.world_point,
                self.squad,
                self.main_weapon.clone(),
                self.magazines.clone(),
                Order::Idle,
                Behavior::Idle(Body::StandUp),
            );
            self.soldiers.push(soldier);
        }

        self
    }

    pub fn collect(&self) -> Vec<SoldierDeployment> {
        self.soldiers.clone()
    }
}

impl Default for ManualSoldiersGenerator {
    fn default() -> Self {
        Self {
            soldiers: vec![],
            type_: SoldierType::Type1,
            side: Side::A,
            squad: SquadUuid(0),
            main_weapon: None,
            magazines: vec![],
            world_point: WorldPoint::new(0., 0.),
        }
    }
}
