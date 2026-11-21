use std::path::PathBuf;

use crate::{
    logging, run,
    utils::{self, Loaded, find_spawn},
};
use anyhow::Context;
use clap::{Parser, ValueEnum};
use oc_deployment::deployment::{self, place::Placer};
use oc_root::{battle::BattlePhase, side::Side};
use oc_world::support::Supports;

const MOD: &str = "mods/std1";
const MAP: &str = "maps/demo2";

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

    let loaded = Loaded::new(&PathBuf::from(MOD), &PathBuf::from(MAP))?;
    let (w, map) = (&loaded.w, &loaded.map);

    let spawns = map.spawn_zones().context("Read map spawn zones")?;
    let deployments = match args.case {
        Case::Empty => deployment::Deployments::default(),
        Case::DeploySouthEst => deployment::Deployments::from_files(
            &PathBuf::from("maps/deployment_allied1.yml"),
            &PathBuf::from("maps/deployment_axis1.yml"),
        )?
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
    let tiles = loaded.tiles(w)?;
    let supports = match args.case {
        Case::Empty => Supports::default(),
        Case::DeploySouthEst => utils::naval_bombards(3),
    };
    let mut snapshot = utils::deployment_snapshot(w, map, tiles.clone(), supports, &deployments)?;

    match args.case {
        Case::Empty => {}
        Case::DeploySouthEst => {
            let places = utils::places(map)?;
            let placer = Placer::new(places.values().copied().collect())
                .with_orders(utils::hide_and_defend(args.side));
            deployments
                .mobilize(w, &loaded.mod_, &mut snapshot, &placer, &tiles)
                .context("Generate snapshot from deployment files")?;
            snapshot.phase = BattlePhase::Deployment;
        }
    }

    run::Example::builder()
        .world(PathBuf::from(MAP))
        .mod_(PathBuf::from(MOD))
        .snapshot(utils::save_snapshot(&snapshot)?)
        .side(args.side)
        .build()
        .run()?;

    Ok(())
}
