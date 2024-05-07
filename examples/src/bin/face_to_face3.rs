use examples::{
    runner::{Runner, RunnerError},
    scenarios::face_to_face::face_to_face,
};

fn main() -> Result<(), RunnerError> {
    let (map, deployment) = face_to_face(50.);

    // FIXME BS NOW disable victory by morale
    Runner::new(map)
        .expire(Some(60 * 60)) // FIXME BS NOW implement
        .deployment(deployment)
        .begin(true)
        .debug_physics(true)
        .run()?;

    Ok(())
}
