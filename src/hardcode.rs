use glam::Vec2;

use crate::{entity::soldier::Soldier, game::Side, types::*, utils};

pub fn get_soldiers() -> Vec<Soldier> {
    let mut soldiers: Vec<Soldier> = vec![];

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

    soldiers
}
