use std::collections::HashMap;

use battle_core::{
    config::TARGET_CYCLE_DURATION_US,
    deployment::{Deployment, SquadTypes},
    game::{
        weapon::{Magazine, Weapon},
        Side,
    },
    types::{SquadUuid, WorldPoint},
};
use battle_gui::debug::DebugTerrain;
use examples::{
    deployment::soldier::ManualSoldiersGenerator,
    map::{flat::Flat, generator::MapGenerator},
    runner::{Runner, RunnerError},
};
use oc_core::game::squad::SquadType;

fn main() -> Result<(), RunnerError> {
    let map = MapGenerator::new(Flat).width(1600).height(150).generate();
    let soldiers = ManualSoldiersGenerator::default()
        .side(Side::A)
        .squad(SquadUuid(0))
        .main_weapon(Some(Weapon::BrenMark2(Some(Magazine::full(
            Magazine::BrenCurved30(0),
        )))))
        .magazines(vec![
            Magazine::full(Magazine::BrenCurved30(0)),
            Magazine::full(Magazine::BrenCurved30(0)),
            Magazine::full(Magazine::BrenCurved30(0)),
            Magazine::full(Magazine::BrenCurved30(0)),
            Magazine::full(Magazine::BrenCurved30(0)),
        ])
        .world_point(WorldPoint::new(25., 25.))
        .place(1, |p: WorldPoint| p)
        .side(Side::B)
        .squad(SquadUuid(1))
        .main_weapon(None)
        .magazines(vec![])
        .world_point(WorldPoint::new(1550., 25.))
        .place(10, |p: WorldPoint| p.apply(WorldPoint::new(0., 5.).into()))
        .collect();
    let mut squad_types = SquadTypes::new();
    squad_types.insert(SquadUuid(0), SquadType::Bren);
    squad_types.insert(SquadUuid(1), SquadType::Type1);
    let deployment = Deployment::new(soldiers, vec![], HashMap::new(), squad_types);

    Runner::new(map)
        .deployment(deployment)
        .begin(true)
        .debug_physics(true)
        .target_cycle_duration(TARGET_CYCLE_DURATION_US)
        .debug_terrain(DebugTerrain::Opacity)
        .run()?;

    Ok(())
}
