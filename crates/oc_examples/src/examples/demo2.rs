#![allow(clippy::unwrap_used)]

use oc_root::format::RkyvFile;
use std::{collections::HashMap, path::PathBuf};

use crate::{logging, run, utils::find_spawn};
use anyhow::Context;
use clap::{Parser, ValueEnum};
use oc_deployment::deployment::{
    self,
    place::{OrderPolicy, Placer},
};
use oc_mod::Mod;
use oc_root::{
    WorldConfig,
    battle::BattlePhase,
    geo::WorldVec2,
    physics::Meters,
    side::{BySide, Side},
};
use oc_world::{
    load::WorldPath,
    meta::Meta,
    place::PlaceName,
    snapshot::Snapshot,
    support::{Support, SupportType, Supports},
};

/// Support declared by mods std1 and tests1
const NAVAL_BOMBARD: &str = "Naval bombard";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: Case,

    #[arg(long, value_enum, default_value = "a")]
    side: Side,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Case {
    Empty,
    DeploySouthEst,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_from(crate::args());
    logging::setup_logging()?;

    let mod_ = Mod::load(&PathBuf::from("mods/std1"), None)?;
    let map_ = PathBuf::from("maps/demo2");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let world = Meta::from_file(&map_.meta());
    let world = world.context(format!("Read file {}", map_.meta().display()))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(world.geo_meters_per_z));
    let map__ = map.build().unwrap();

    let spawns = map.spawn_zones().context("Read map spawn zones")?;
    let deployments = match args.case {
        Case::Empty => deployment::Deployments::default(),
        Case::DeploySouthEst => deployment::Deployments::from_files(
            &PathBuf::from("maps/deployment_allied1.yml"),
            &PathBuf::from("maps/deployment_axis1.yml"),
        )
        .unwrap()
        .with_spawns(
            vec![
                find_spawn(&spawns, "SouthEast"),
                find_spawn(&spawns, "South"),
            ],
            vec![
                find_spawn(&spawns, "North"),
                find_spawn(&spawns, "NorthWest"),
                find_spawn(&spawns, "West"),
                find_spawn(&spawns, "CenterForest"),
                find_spawn(&spawns, "WestTown"),
                find_spawn(&spawns, "EstTown"),
                find_spawn(&spawns, "SouthWest"),
            ],
        ),
    };
    let tiles = map.tiles(&w, &mod_).context("Read map tiles")?;
    let places: HashMap<PlaceName, WorldVec2> = map__
        .places()
        .iter()
        .map(|p| (p.name.clone(), WorldVec2::new(p.x, p.y)))
        .collect();
    let mut snapshot = Snapshot::empty(w.clone())
        .with_tiles(tiles.clone())
        .with_decor(map.decor().context("Read map decor")?)
        .with_supports(match args.case {
            Case::Empty => Supports::default(),
            Case::DeploySouthEst => Supports::new(
                Support(vec![(SupportType::new(NAVAL_BOMBARD), 3)]),
                Support::default(),
            ),
        });

    snapshot.spawns = deployments
        .as_ref()
        .map(|d| d.spawns.iter().map(|s| s.name.clone()).collect());

    match args.case {
        Case::Empty => {}
        Case::DeploySouthEst => {
            deploy_south_est(
                mod_,
                w,
                args.side,
                deployments,
                tiles,
                places,
                &mut snapshot,
            )?;
        }
    }

    let (_, snapshot_path) = tempfile::NamedTempFile::new()?.keep()?;
    snapshot
        .save(&snapshot_path)
        .context(format!("Save snapshot to {}", snapshot_path.display()))?;

    let example = run::Example::builder()
        .world(map_)
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot_path)
        .side(args.side);
    example.build().run()?;

    Ok(())
}

fn deploy_south_est(
    mod_: Mod,
    w: WorldConfig,
    side: Side,
    deployments: deployment::Deployments,
    tiles: Vec<oc_world::tile::Tile>,
    places: HashMap<PlaceName, WorldVec2>,
    snapshot: &mut Snapshot,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let places_ = places.values().copied();
    let placer = Placer::new(places_.collect()).with_assigned(match side {
        Side::A => vec![],
        Side::B => vec![],
    });
    let placer = match side {
        Side::A => placer.with_orders(BySide::new(OrderPolicy::Hide, OrderPolicy::Defend)),
        Side::B => placer.with_orders(BySide::new(OrderPolicy::Defend, OrderPolicy::Hide)),
    };

    deployments
        .mobilize(&w, &mod_, snapshot, &placer, &tiles)
        .context("Generate snapshot from deployment files".to_string())?;

    snapshot.phase = BattlePhase::Deployment;

    Ok(())
}
