use glam::Vec2;

use crate::{
    entity::{
        soldier::Soldier,
        vehicle::{OnBoardPlace, Vehicle, VehicleType},
    },
    game::Side,
    types::*,
    utils,
};

pub fn shared_state_fixtures() -> (Vec<Soldier>, Vec<Vehicle>, SoldiersOnBoard) {
    let mut soldiers = vec![];
    let mut vehicles = vec![];
    let mut boards = SoldiersOnBoard::new();

    for x in 0..1 {
        let squad = utils::new_squad_uuid();
        for y in 0..10 {
            // let x: f32 = rng.gen_range(0.0..800.0);
            // let y: f32 = rng.gen_range(0.0..800.0);
            let soldier = Soldier::new(
                WorldPoint::from(Vec2::new(x as f32 * 10., y as f32 * 10.)),
                SquadUuid(squad),
                Side::A,
            );
            soldiers.push(soldier);
        }
    }

    let tank = Vehicle::new(VehicleType::T26, WorldPoint::from(Vec2::new(100., 100.)));
    vehicles.push(tank);

    let tank1_squad = utils::new_squad_uuid();
    let tank_driver = Soldier::new(
        WorldPoint::from(Vec2::new(0., 0.)),
        SquadUuid(tank1_squad),
        Side::A,
    );
    soldiers.push(tank_driver);
    boards.insert(
        SoldierIndex(soldiers.len() - 1),
        (VehicleIndex(vehicles.len() - 1), OnBoardPlace::Driver),
    );
    let tank_gunner = Soldier::new(
        WorldPoint::from(Vec2::new(0., 0.)),
        SquadUuid(tank1_squad),
        Side::A,
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
