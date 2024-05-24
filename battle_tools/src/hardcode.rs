use std::collections::HashMap;

use battle_core::{
    behavior::{Behavior, Body},
    deployment::{Deployment, SoldierDeployment, VehicleDeployment},
    entity::vehicle::{OnBoardPlace, VehicleType},
    game::{
        weapon::{Magazine, Weapon},
        Side,
    },
    order::Order,
    types::{SoldierIndex, SoldiersOnBoard, SquadUuid, VehicleIndex, WorldPoint},
    utils,
};
use glam::Vec2;
use oc_core::game::{soldier::SoldierType, squad::SquadType};

pub fn demo1_deployment() -> Deployment {
    let mut soldiers = vec![];
    let mut vehicles = vec![];
    let mut boards = SoldiersOnBoard::new();
    let mut squad_types = HashMap::new();
    let mut soldiers_index: usize = 0;

    for x in 0..5 {
        let squad = SquadUuid(utils::new_squad_uuid());
        squad_types.insert(squad, SquadType::Type1);
        for y in 0..5 {
            // let x: f32 = rng.gen_range(0.0..800.0);
            // let y: f32 = rng.gen_range(0.0..800.0);
            let soldier = SoldierDeployment::new(
                SoldierIndex(soldiers_index),
                SoldierType::Type1,
                Side::A,
                WorldPoint::from(Vec2::new(x as f32 * 10. + 20.0, y as f32 * 10. + 100.)),
                squad,
                Some(Weapon::MosinNagantM1924(
                    false,
                    Some(Magazine::full(Magazine::MosinNagant(0))),
                )),
                vec![
                    Magazine::full(Magazine::MosinNagant(0)),
                    Magazine::full(Magazine::MosinNagant(0)),
                ],
                Order::Idle,
                Behavior::Idle(Body::StandUp),
            );
            soldiers.push(soldier);
            soldiers_index += 1;
        }
    }

    for x in 0..4 {
        let squad = SquadUuid(utils::new_squad_uuid());
        squad_types.insert(squad, SquadType::Type1);
        for y in 0..5 {
            // let x: f32 = rng.gen_range(0.0..800.0);
            // let y: f32 = rng.gen_range(0.0..800.0);
            let soldier = SoldierDeployment::new(
                SoldierIndex(soldiers_index),
                SoldierType::Type1,
                Side::B,
                WorldPoint::from(Vec2::new(x as f32 * 10. + 550., y as f32 * 10. + 250.)),
                squad,
                Some(Weapon::MosinNagantM1924(
                    false,
                    Some(Magazine::full(Magazine::MosinNagant(0))),
                )),
                vec![
                    Magazine::full(Magazine::MosinNagant(0)),
                    Magazine::full(Magazine::MosinNagant(0)),
                ],
                Order::Idle,
                Behavior::Idle(Body::StandUp),
            );
            soldiers.push(soldier);
            soldiers_index += 1;
        }
    }

    let tank = VehicleDeployment::new(
        VehicleIndex(0),
        VehicleType::T26,
        WorldPoint::from(Vec2::new(100., 100.)),
    );
    vehicles.push(tank);

    let tank1_squad = SquadUuid(utils::new_squad_uuid());
    squad_types.insert(tank1_squad, SquadType::Type1);
    let tank_driver = SoldierDeployment::new(
        SoldierIndex(soldiers_index),
        SoldierType::Type1,
        Side::A,
        WorldPoint::from(Vec2::new(0., 0.)),
        tank1_squad,
        None,
        vec![],
        Order::Idle,
        Behavior::Idle(Body::StandUp),
    );
    soldiers.push(tank_driver);
    soldiers_index += 1;
    boards.insert(
        SoldierIndex(soldiers.len() - 1),
        (VehicleIndex(vehicles.len() - 1), OnBoardPlace::Driver),
    );
    let tank_gunner = SoldierDeployment::new(
        SoldierIndex(soldiers_index),
        SoldierType::Type1,
        Side::A,
        WorldPoint::from(Vec2::new(0., 0.)),
        tank1_squad,
        None,
        vec![],
        Order::Idle,
        Behavior::Idle(Body::StandUp),
    );
    soldiers.push(tank_gunner);
    boards.insert(
        SoldierIndex(soldiers.len() - 1),
        (
            VehicleIndex(vehicles.len() - 1),
            OnBoardPlace::MainTurretGunner,
        ),
    );

    Deployment::from((soldiers, vehicles, boards, squad_types))
}
