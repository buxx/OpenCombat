use std::collections::HashMap;

use crate::{
    deployment::soldier::ManualSoldiersGenerator,
    map::{flat::Flat, generator::MapGenerator, MapModel},
};
use battle_core::{
    deployment::{Deployment, SquadTypes},
    game::{
        weapon::{Magazine, Weapon},
        Side,
    },
    map::{terrain::TileType, Map},
    types::{GridPoint, SquadUuid, WorldPoint},
};
use oc_core::game::squad::SquadType;

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
        Magazine::full(Magazine::Mauser(0)),
        Magazine::full(Magazine::Mauser(0)),
    ]
}

pub fn face_to_face(
    default_tile_type: TileType,
    distance: f32,
    hide: Option<TileType>,
) -> (Map, Deployment) {
    let original_x = 75.;
    let original_y = 50.;
    let y_increment = 5.;
    let squad_members = 5;

    let place = if let Some(tile_type) = hide {
        let mut place = vec![];
        for i in 0..squad_members {
            let x = original_x + distance;
            let y = original_y + y_increment * i as f32;
            let column = x as u32 / Flat.terrain_tile_size();
            let line = y as u32 / Flat.terrain_tile_size();
            place.push((
                GridPoint::new(column as i32, line as i32),
                tile_type.clone(),
            ));
        }
        place
    } else {
        vec![]
    };

    let map = MapGenerator::new(Flat)
        .width(1600)
        .height(150)
        .default_tile_type(default_tile_type)
        .place(place)
        .generate();
    let soldiers = ManualSoldiersGenerator::default()
        .side(Side::A)
        .squad(SquadUuid(0))
        .main_weapon(Some(mosin_nagant()))
        .magazines(mosin_nagant_magazines())
        .world_point(WorldPoint::new(original_x, original_y))
        .place(squad_members, |p: WorldPoint| {
            p.apply(WorldPoint::new(0., y_increment).into())
        })
        .side(Side::B)
        .squad(SquadUuid(1))
        .main_weapon(Some(mauser()))
        .magazines(mauser_magazines())
        .world_point(WorldPoint::new(original_x + distance, original_y))
        .place(squad_members, |p: WorldPoint| {
            p.apply(WorldPoint::new(0., y_increment).into())
        })
        .collect();

    let mut squad_types = SquadTypes::new();
    squad_types.insert(SquadUuid(0), SquadType::Type1);
    squad_types.insert(SquadUuid(1), SquadType::Type1);

    (
        map,
        Deployment::new(soldiers, vec![], HashMap::new(), squad_types),
    )
}
