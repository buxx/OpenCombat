use crate::{entity::soldier::Soldier, types::*, utils};
use rand::Rng;

pub fn get_entities() -> Vec<ThreadSafeEntity> {
    let mut entities: Vec<ThreadSafeEntity> = vec![];
    let mut rng = rand::thread_rng();

    for x in 0..10 {
        for y in 0..10 {
            // TODO: for now, one entity by squad
            let squad = utils::squad_uuid();
            // let x: f32 = rng.gen_range(0.0..800.0);
            // let y: f32 = rng.gen_range(0.0..800.0);
            let entity = Box::new(Soldier::new(
                WorldPosition::from((WorldX::from(x as f32 * 10.), WorldY::from(y as f32 * 10.))),
                squad,
            ));
            entities.push(entity);
        }
    }

    entities
}
