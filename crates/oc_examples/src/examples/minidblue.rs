use std::{collections::HashMap, path::PathBuf};

use crate::{
    logging, run,
    utils::{self, Loaded, find_spawn, squad_at},
};
use anyhow::Context;
use clap::{Parser, ValueEnum};
use oc_deployment::deployment::{
    self,
    place::{OrderPolicy, Placer},
};
use oc_root::{
    battle::BattlePhase,
    geo::WorldVec2,
    side::{BySide, Side},
};
use oc_world::{place::PlaceName, support::Supports};

const MOD: &str = "mods/std1";
const MAP: &str = "maps/minidblue";

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
    EnagedBattleMiddle,
    DeploySouthBeach,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_from(crate::args());
    logging::setup_logging()?;

    let loaded = Loaded::new(&PathBuf::from(MOD), &PathBuf::from(MAP))?;
    let (w, map) = (&loaded.w, &loaded.map);

    let spawns = map.spawn_zones().context("Read map spawn zones")?;
    let deployments = match args.case {
        Case::Empty => deployment::Deployments::default(),
        Case::EnagedBattleMiddle => deployment::Deployments::from_files(
            &PathBuf::from("examples/deployment1a.yml"),
            &PathBuf::from("examples/deployment1b.yml"),
        )?,
        Case::DeploySouthBeach => deployment::Deployments::from_files(
            &PathBuf::from("examples/deployment_allied1.yml"),
            &PathBuf::from("examples/deployment_axis1.yml"),
        )?
        .with_spawns(
            vec![find_spawn(&spawns, "SouthBeach")],
            vec![find_spawn(&spawns, "Middle"), find_spawn(&spawns, "North")],
        ),
    };
    let tiles = loaded.tiles(w)?;
    let supports = match args.case {
        Case::Empty | Case::EnagedBattleMiddle => Supports::default(),
        Case::DeploySouthBeach => utils::naval_bombards(3),
    };
    let mut snapshot = utils::deployment_snapshot(w, map, tiles.clone(), supports, &deployments)?;

    let placed = match args.case {
        Case::Empty => None,
        Case::EnagedBattleMiddle => Some((
            engaged_battle_middle(&utils::places(map)?),
            BattlePhase::Fight,
        )),
        Case::DeploySouthBeach => Some((
            south_beach(&utils::places(map)?, args.side),
            BattlePhase::Deployment,
        )),
    };
    if let Some((placer, phase)) = placed {
        deployments
            // FIXME BS NOW: 1/ Début de partie ne doit pas impliquer mouvement de membres pour se place.
            //                  Le placer doit placer les individus a leur place directement.
            //               2/ Les soldats ne doivent pas tirer dès le début car hide. Là il y a des tirs.
            //                  Maybe individual n'ont pas encore reçu order (placer doit donner order)
            //                  Sinon c'est du tuning de behavior.
            // Sinon, créer les mécanique de la phase "deployment"
            .mobilize(w, &loaded.mod_, &mut snapshot, &placer, &tiles)
            .context("Generate snapshot from deployment files")?;
        snapshot.phase = phase;
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

/// Hand made placer, to place squads manually
fn engaged_battle_middle(places: &HashMap<PlaceName, WorldVec2>) -> Placer {
    Placer::new(vec![])
        .with_assigned(vec![
            // uuids are these from examples/deployment1a.yml & examples/deployment1b.yml
            // places names are from examples/minidblue
            // A
            squad_at(places, "f9dbffe2-64ba-4a88-920b-ae883751493e", "63"),
            squad_at(places, "ca3db430-f7fa-4232-b2a1-2abb85499ae5", "50"),
            // B
            squad_at(places, "e1b6ec3d-7119-41d1-9f89-a6faf7da5bbb", "70"),
            squad_at(places, "9dbf54c7-3723-4fe3-a42b-44f3866c09aa", "53"),
        ])
        .with_orders(BySide::new(OrderPolicy::Hide, OrderPolicy::Defend))
}

fn south_beach(places: &HashMap<PlaceName, WorldVec2>, side: Side) -> Placer {
    Placer::new(places.values().copied().collect())
        .with_assigned(match side {
            // B
            Side::A => vec![
                squad_at(places, "53f27839-4f3f-486e-bb47-420f382be19c", "50"), // Mg34
                squad_at(places, "03f99100-deff-4bd6-9b24-d3a4e1b3bbf6", "56"), // Mg34
                squad_at(places, "04f4eec9-fc48-4bf9-a2cd-4d072656e4a7", "42"), // Rifle
                squad_at(places, "578150d7-4cd9-448f-82cb-44c70e1da22b", "49"), // Rifle
            ],
            Side::B => vec![],
        })
        .with_orders(utils::hide_and_defend(side))
}
