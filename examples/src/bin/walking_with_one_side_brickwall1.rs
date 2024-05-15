use battle_core::{
    config::TARGET_CYCLE_DURATION_US,
    map::terrain::TileType,
    order::Order,
    state::battle::message::{BattleStateMessage, SoldierMessage},
    types::{Angle, SoldierIndex, WorldPath, WorldPaths, WorldPoint},
};
use battle_gui::{debug::DebugTerrain, engine::message::EngineMessage};
use examples::{
    runner::{Runner, RunnerError},
    scenarios::face_to_face::face_to_face,
};

fn main() -> Result<(), RunnerError> {
    let (map, deployment) = face_to_face(TileType::ShortGrass, 350., Some(TileType::BrickWall));

    let messages = vec![
        EngineMessage::BattleState(BattleStateMessage::Soldier(
            SoldierIndex(0),
            SoldierMessage::SetOrder(Order::MoveTo(
                WorldPaths::new(vec![WorldPath::new(vec![WorldPoint::new(500., 50.)])]),
                None,
            )),
        )),
        EngineMessage::BattleState(BattleStateMessage::Soldier(
            SoldierIndex(5),
            SoldierMessage::SetOrder(Order::Hide(Angle(0.75))),
        )),
    ];

    // FIXME BS NOW: side B is not hided at start, why ?
    Runner::new(map)
        .deployment(deployment)
        .begin(true)
        .debug_physics(true)
        .target_cycle_duration(TARGET_CYCLE_DURATION_US / 3)
        .debug_terrain(DebugTerrain::Opacity)
        .when_first_copy_apply(messages)
        .run()?;

    Ok(())
}
