use crate::{
    entity::{
        soldier::Soldier,
        vehicle::{OnBoardPlace, Vehicle, VehicleType},
    },
    game::{
        weapon::{Magazine, Weapon},
        Side,
    },
    types::{SoldierIndex, SoldiersOnBoard, SquadUuid, VehicleIndex, WorldPoint},
    utils,
};
use glam::Vec2;

pub fn situation() -> (Vec<Soldier>, Vec<Vehicle>, SoldiersOnBoard) {
    let mut soldiers = vec![];
    let mut vehicles = vec![];
    let mut boards = SoldiersOnBoard::new();
    let mut soldiers_index: usize = 0;

    for x in 0..1 {
        let squad = utils::new_squad_uuid();
        for y in 0..5 {
            // let x: f32 = rng.gen_range(0.0..800.0);
            // let y: f32 = rng.gen_range(0.0..800.0);
            let soldier = Soldier::new(
                SoldierIndex(soldiers_index),
                WorldPoint::from(Vec2::new(x as f32 * 10. + 20.0, y as f32 * 10. + 100.)),
                SquadUuid(squad),
                Side::A,
                Some(Weapon::MosinNagantM1924(
                    false,
                    Some(Magazine::full(Magazine::MosinNagant(0))),
                )),
                vec![
                    Magazine::full(Magazine::MosinNagant(0)),
                    Magazine::full(Magazine::MosinNagant(0)),
                ],
            );
            soldiers.push(soldier);
            soldiers_index += 1;
        }
    }

    for x in 0..1 {
        let squad = utils::new_squad_uuid();
        for y in 0..5 {
            // let x: f32 = rng.gen_range(0.0..800.0);
            // let y: f32 = rng.gen_range(0.0..800.0);
            let soldier = Soldier::new(
                SoldierIndex(soldiers_index),
                WorldPoint::from(Vec2::new(x as f32 * 10. + 550., y as f32 * 10. + 250.)),
                SquadUuid(squad),
                Side::B,
                Some(Weapon::MosinNagantM1924(
                    false,
                    Some(Magazine::full(Magazine::MosinNagant(0))),
                )),
                vec![
                    Magazine::full(Magazine::MosinNagant(0)),
                    Magazine::full(Magazine::MosinNagant(0)),
                ],
            );
            soldiers.push(soldier);
            soldiers_index += 1;
        }
    }

    let tank = Vehicle::new(
        VehicleIndex(0),
        VehicleType::T26,
        WorldPoint::from(Vec2::new(100., 100.)),
    );
    vehicles.push(tank);

    let tank1_squad = utils::new_squad_uuid();
    let tank_driver = Soldier::new(
        SoldierIndex(soldiers_index),
        WorldPoint::from(Vec2::new(0., 0.)),
        SquadUuid(tank1_squad),
        Side::A,
        None,
        vec![],
    );
    soldiers.push(tank_driver);
    soldiers_index += 1;
    boards.insert(
        SoldierIndex(soldiers.len() - 1),
        (VehicleIndex(vehicles.len() - 1), OnBoardPlace::Driver),
    );
    let tank_gunner = Soldier::new(
        SoldierIndex(soldiers_index),
        WorldPoint::from(Vec2::new(0., 0.)),
        SquadUuid(tank1_squad),
        Side::A,
        None,
        vec![],
    );
    soldiers.push(tank_gunner);
    boards.insert(
        SoldierIndex(soldiers.len() - 1),
        (
            VehicleIndex(vehicles.len() - 1),
            OnBoardPlace::MainTurretGunner,
        ),
    );

    (soldiers, vehicles, boards)
}