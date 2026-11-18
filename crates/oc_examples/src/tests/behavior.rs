use oc_utils::index::AtIndex;
use std::path::PathBuf;

use anyhow::Context as ContextExt;
use bevy::prelude::*;
use bon::builder;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::{
    BodyGesture, Gesture, IndividualIndex, Weapons,
    order::Order,
    squad::{Squad, SquadFormation},
};
use oc_mod::{grenade::GrenadeIndex, illustration::IllustrationIndex};
use oc_root::{
    WcfgFrom, WorldConfig, battle::BattlePhase, behavior::Stun, physics::Meters, side::Side, y::V,
};
use oc_utils::d2::{Direction, Xy};
use oc_world::{meta::Meta, tile::Tile};
#[cfg(feature = "test")]
use oc_world_server::tracker::Tracker;

use crate::{run, snapshot::SnapshotBuilder};

type Install = Box<dyn Fn(&mut bevy::app::App)>;
#[cfg(feature = "test")]
type Track = Box<dyn Fn(Tracker)>;
#[cfg(not(feature = "test"))]
type Track = ();

const DIRECTION: Direction = Direction::SOUTH;

/// Grenade (from tests mod) given to individuals
pub const FRAG_GRENADE: &str = "Frag grenade";

#[builder]
pub fn run(
    setup: Vec<([f32; 2], Vec<Order>)>,
    individual_count: usize,
    /// Number of frag grenades carried by each individual
    grenade_count: usize,
    /// Alone (in their squad) side B individuals, without grenade
    #[builder(default)]
    enemies: Vec<([f32; 2], Vec<Order>)>,
    tests: (Install, Track),
    test: bool,
) -> Result<(), anyhow::Error> {
    let mod_ = PathBuf::from("mods/tests1");
    let mod__ = oc_mod::Mod::load(&mod_, None)?;
    let map = PathBuf::from("maps/meadow1");
    let meta = Meta::from_file(&map.join("meta.toml"))?;
    let map_ = oc_world::reader::MapReader::new(&map);
    let map_ = map_.context(format!("Read map_ {}", map.display()))?;
    let w = WorldConfig::new(map_.width().unwrap() as u64, map_.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z))
        // Shorter stun than default, to keep examples fast
        .with_individual_tick_decrease_stunned(Stun(51));
    let tiles = map_.tiles(&w, &mod__).unwrap();

    let frag = mod__.grenade_from_name(FRAG_GRENADE)?.index();
    let grenades = vec![frag; grenade_count];
    let mut individuals = individuals(&w, &tiles, &setup, individual_count, &grenades);
    let mut squads = squads(&setup, individual_count);
    for (position, orders) in &enemies {
        let i = IndividualIndex(individuals.len() as u64);
        individuals.push(individual(&w, &tiles, Side::B, *position, &[]));
        squads.push(squad(Side::B, *position, orders, vec![i]));
    }
    let snapshot = SnapshotBuilder::new(
        map_,
        individuals,
        squads,
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .test_app_exit_code(test)
        .snapshot(snapshot);

    let example = example.install(tests.0);

    #[cfg(feature = "test")]
    let tracker = example.build().run()?;
    #[cfg(not(feature = "test"))]
    example.build().run()?;

    #[cfg(feature = "test")]
    {
        tests.1(tracker);
    }

    Ok(())
}

fn individuals(
    w: &WorldConfig,
    tiles: &[Tile],
    setup: &[([f32; 2], Vec<Order>)],
    count: usize,
    grenades: &[GrenadeIndex],
) -> Vec<oc_individual::Individual> {
    setup
        .iter()
        .map(|(position, orders)| {
            let positions = SquadFormation::Line.positions(
                w,
                V::Server,
                (*position).into(),
                DIRECTION.angle(V::Server),
                count,
            );
            (position, orders, positions)
        })
        .flat_map(|(_, _, positions)| {
            positions
                .into_iter()
                .map(|position| individual(w, tiles, Side::A, [position.x, position.y], grenades))
        })
        .collect()
}

fn individual(
    w: &WorldConfig,
    tiles: &[Tile],
    side: Side,
    position: [f32; 2],
    grenades: &[GrenadeIndex],
) -> oc_individual::Individual {
    let tile_xy = TileXy(Xy(
        position[0] as u64 / w.geo_pixels_per_tile(),
        position[1] as u64 / w.geo_pixels_per_tile(),
    ));
    let tile_i = WorldTileIndex::from_(tile_xy, w);
    let z = tiles.at(tile_i).z_pixels(w);

    oc_individual::Individual::fresh(w, side, [position[0], position[1], z].into())
        .with_gesture(Gesture::body(BodyGesture::StandUp(DIRECTION)))
        .with_weapons(Weapons {
            primary: None,
            grenades: grenades.to_vec(),
        })
}

fn squads(setup: &[([f32; 2], Vec<Order>)], count: usize) -> Vec<Squad> {
    // For this test, all individual are alone in their squad
    // Test of squad behavior is in other example
    setup
        .iter()
        .map(|(position, orders)| {
            let members = (0..count).map(IndividualIndex::from).collect();
            squad(Side::A, *position, orders, members)
        })
        .collect()
}

fn squad(side: Side, position: [f32; 2], orders: &[Order], members: Vec<IndividualIndex>) -> Squad {
    Squad {
        side,
        illustration: IllustrationIndex(0),
        position: position.into(),
        leader: members[0],
        actives: members.len() as u8,
        members,
        formation: SquadFormation::Line,
        orders: orders.to_vec(),
        status: oc_individual::squad::SquadStatus::Operational,
    }
}
