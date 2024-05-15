use battle_core::{config::TARGET_CYCLE_DURATION_US, map::terrain::TileType};
use battle_gui::debug::DebugTerrain;
use examples::{
    runner::{Runner, RunnerError},
    scenarios::face_to_face::face_to_face,
};

fn main() -> Result<(), RunnerError> {
    let (map, deployment) = face_to_face(TileType::ShortGrass, 500., Some(TileType::BrickWall));

    Runner::new(map)
        .deployment(deployment)
        .begin(true)
        .debug_physics(true)
        .target_cycle_duration(TARGET_CYCLE_DURATION_US / 3)
        .debug_terrain(DebugTerrain::Opacity)
        .run()?;

    Ok(())
}
