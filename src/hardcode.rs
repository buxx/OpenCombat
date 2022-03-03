use crate::{entity::soldier::Soldier, types::*, utils};
use rand::Rng;

pub fn get_entities() -> Vec<ThreadSafeEntity> {
    let mut entities: Vec<ThreadSafeEntity> = vec![];
    let mut rng = rand::thread_rng();

    for _ in 0..20 {
        // TODO: for now, one entity by squad
        let squad = utils::squad_uuid();
        let x: f32 = rng.gen_range(0.0..800.0);
        let y: f32 = rng.gen_range(0.0..800.0);
        let entity = Box::new(Soldier::new(
            WorldPosition::from((WorldX::from(x), WorldY::from(y))),
            squad,
        ));
        entities.push(entity);
    }

    entities
}
