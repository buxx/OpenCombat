use std::collections::HashMap;

use battle_core::{
    deployment::Deployment,
    game::{
        weapon::{Magazine, Weapon},
        Side,
    },
    types::{SquadUuid, WorldPoint},
};
use examples::{
    deployment::soldier::ManualSoldiersGenerator,
    map::{flat::FlatAndEmpty, generator::MapGenerator},
    runner::{Runner, RunnerError},
};

fn mosin_nagant() -> Weapon {
    Weapon::MosinNagantM1924(true, Some(Magazine::full(Magazine::MosinNagant(0))))
}

fn mosin_nagant_magazines() -> Vec<Magazine> {
    vec![
        Magazine::full(Magazine::MosinNagant(0)),
        Magazine::full(Magazine::MosinNagant(0)),
        Magazine::full(Magazine::MosinNagant(0)),
        Magazine::full(Magazine::MosinNagant(0)),
    ]
}

fn mauser() -> Weapon {
    Weapon::MauserG41(true, Some(Magazine::full(Magazine::Mauser(0))))
}

fn mauser_magazines() -> Vec<Magazine> {
    vec![
        Magazine::full(Magazine::Mauser(0)),
        Magazine::full(Magazine::Mauser(0)),
        Magazine::full(Magazine::Mauser(0)),
        Magazine::full(Magazine::Mauser(0)),
    ]
}

fn main() -> Result<(), RunnerError> {
    let map = MapGenerator::new(FlatAndEmpty)
        .width(500)
        .height(150)
        .generate();
    let soldiers = ManualSoldiersGenerator::default()
        .side(Side::A)
        .squad(SquadUuid(0))
        .main_weapon(Some(mosin_nagant()))
        .magazines(mosin_nagant_magazines())
        .world_point(WorldPoint::new(75.0, 50.0))
        .place(5, |p: WorldPoint| p.apply(WorldPoint::new(0., 5.).into()))
        .side(Side::B)
        .squad(SquadUuid(1))
        .main_weapon(Some(mauser()))
        .magazines(mauser_magazines())
        .world_point(WorldPoint::new(325.0, 50.0))
        .place(5, |p: WorldPoint| p.apply(WorldPoint::new(0., 5.).into()))
        .collect();
    let deployment = Deployment::new(soldiers, vec![], HashMap::new());

    Runner::new(map)
        .expire(Some(60 * 60))
        .deployment(deployment)
        .run()?;

    Ok(())
}
