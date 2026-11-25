use std::path::PathBuf;

use crate::{
    logging, run,
    utils::{self, Loaded, find_spawn},
};
use anyhow::Context;
use clap::{Parser, ValueEnum};
use oc_battle::{Battle, Scenario, SquadPlace};
use oc_deployment::{
    deployment::{self, Deployment, Deployments, place::Placer},
    generator::Generator,
};
use oc_mod::Mod;
use oc_root::{
    battle::BattlePhase,
    geo::WorldVec2,
    side::{BySide, Side},
    uuids::SquadUuid,
};
use oc_world::{place::PlaceName, spawn::SpawnZone, support::Supports};
use std::collections::HashMap;

const MOD: &str = "mods/std1";
const MAP: &str = "maps/demo2";
const MAPS_FOLDER: &str = "maps";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: Case,

    #[arg(long, value_enum, default_value = "a")]
    side: Side,

    /// Battle file (like `battles/Orchard_battle/Orchard_battle.ron`), required by `load-battle`
    #[arg(long, required_if_eq("case", "load-battle"))]
    battle_path: Option<PathBuf>,

    /// Scenario file (like `battles/Orchard_battle/Allied_push.ron`), required by `load-battle`
    #[arg(long, required_if_eq("case", "load-battle"))]
    scenario_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Case {
    Empty,
    DeploySouthEst,
    /// Battle (map) and scenario loaded from files, started with scenario default deployment
    LoadBattle,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_from(crate::args());
    logging::setup_logging()?;

    let battle = match args.case {
        Case::LoadBattle => Some(load_battle(&args)?),
        Case::Empty | Case::DeploySouthEst => None,
    };
    let map_path = battle
        .as_ref()
        .map(|(battle, _)| PathBuf::from(MAPS_FOLDER).join(battle.map()))
        .unwrap_or_else(|| PathBuf::from(MAP));

    let loaded = Loaded::new(&PathBuf::from(MOD), &map_path)?;
    let (w, map) = (&loaded.w, &loaded.map);

    let spawns = map.spawn_zones().context("Read map spawn zones")?;
    let deployments = match (args.case, &battle) {
        (Case::Empty, _) => deployment::Deployments::default(),
        (Case::DeploySouthEst, _) => deployment::Deployments::from_files(
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
        (Case::LoadBattle, Some((_, scenario))) => {
            scenario_deployments(&loaded.mod_, scenario, &spawns)?
        }
        (Case::LoadBattle, None) => unreachable!("Battle loaded for LoadBattle case"),
    };
    let tiles = loaded.tiles(w)?;
    let supports = match (args.case, &battle) {
        (Case::Empty, _) => Supports::default(),
        (Case::DeploySouthEst, _) => utils::naval_bombards(3),
        (Case::LoadBattle, Some((_, scenario))) => Supports::new(
            scenario.side(Side::A).supports.clone(),
            scenario.side(Side::B).supports.clone(),
        ),
        (Case::LoadBattle, None) => unreachable!("Battle loaded for LoadBattle case"),
    };
    let mut snapshot = utils::deployment_snapshot(w, map, tiles.clone(), supports, &deployments)?;

    match (args.case, &battle) {
        (Case::Empty, _) => {}
        (Case::DeploySouthEst, _) => {
            let places = utils::places(map)?;
            let placer = Placer::new(places.values().copied().collect())
                .with_orders(utils::hide_and_defend(args.side));
            deployments
                .mobilize(w, &loaded.mod_, &mut snapshot, &placer, &tiles)
                .context("Generate snapshot from deployment files")?;
            snapshot.phase = BattlePhase::Deployment;
        }
        (Case::LoadBattle, Some((_, scenario))) => {
            let places = utils::places(map)?;
            let default_orders = utils::hide_and_defend(args.side);
            let placer = Placer::new(places.values().copied().collect())
                .with_assigned(scenario_places(&deployments, scenario, &places)?)
                .with_orders(BySide::from_fn(|side| {
                    scenario.side(side).order.unwrap_or(default_orders[side])
                }));
            deployments
                .mobilize(w, &loaded.mod_, &mut snapshot, &placer, &tiles)
                .context("Generate snapshot from scenario deployments")?;
            snapshot.phase = BattlePhase::Deployment;
        }
        (Case::LoadBattle, None) => unreachable!("Battle loaded for LoadBattle case"),
    }

    run::Example::builder()
        .world(map_path)
        .mod_(PathBuf::from(MOD))
        .snapshot(utils::save_snapshot(&snapshot)?)
        .side(args.side)
        .build()
        .run()?;

    Ok(())
}

fn load_battle(args: &Args) -> anyhow::Result<(Battle, Scenario)> {
    let battle_path = args
        .battle_path
        .as_deref()
        .context("--battle-path required")?;
    let scenario_path = args
        .scenario_path
        .as_deref()
        .context("--scenario-path required")?;
    let battle = Battle::from_ron_file(battle_path)
        .context(format!("Read battle {}", battle_path.display()))?;
    let scenario = Scenario::from_ron_file(scenario_path)
        .context(format!("Read scenario {}", scenario_path.display()))?;
    Ok((battle, scenario))
}

/// Deployments generated from scenario sides profiles, keeping only the scenario default squads,
/// in scenario start spawn zones
fn scenario_deployments(
    mod_: &Mod,
    scenario: &Scenario,
    spawns: &[SpawnZone],
) -> anyhow::Result<Deployments> {
    let generator = Generator::new();
    let deployment = |side: Side| -> anyhow::Result<Deployment> {
        let scenario_side = scenario.side(side);
        let generated = generator
            .generate(mod_, &scenario_side.deploy)
            .context(format!("Generate side {side:?} deployment"))?;
        Ok(Deployment {
            squads: default_squads(generated, &scenario_side.default, scenario_side.squad_count),
            spawns: scenario_side
                .start_spawn_zones
                .iter()
                .map(|name| find_spawn(spawns, &name.0))
                .collect(),
        })
    };
    Ok(Deployments(BySide::new(
        deployment(Side::A)?,
        deployment(Side::B)?,
    )))
}

/// First not yet taken squads matching `defaults` labels (at most `count`)
fn default_squads(
    deployment: Deployment,
    defaults: &[String],
    count: usize,
) -> Vec<deployment::squad::Squad> {
    let mut available: Vec<_> = deployment.squads.into_iter().map(Some).collect();
    defaults
        .iter()
        .take(count)
        .filter_map(|label| {
            available
                .iter_mut()
                .find(|squad| squad.as_ref().is_some_and(|s| s.label == *label))
                .and_then(Option::take)
        })
        .collect()
}

/// Scenario squads places (by squad label and place name) assigned to deployments squads
fn scenario_places(
    deployments: &Deployments,
    scenario: &Scenario,
    places: &HashMap<PlaceName, WorldVec2>,
) -> anyhow::Result<Vec<(SquadUuid, WorldVec2)>> {
    let mut assigned = vec![];
    for (side, deployment) in deployments.iter() {
        let mut taken = vec![false; deployment.squads.len()];
        for place in &scenario.side(side).places {
            let Some(index) = squad_for(deployment, &taken, place) else {
                tracing::warn!(
                    "No side {side:?} squad '{}' for place '{}'",
                    place.squad(),
                    place.place()
                );
                continue;
            };
            taken[index] = true;
            let position = places
                .get(&place.place().into())
                .context(format!("Place '{}' not found in map", place.place()))?;
            assigned.push((deployment.squads[index].uuid, *position));
        }
    }
    Ok(assigned)
}

fn squad_for(deployment: &Deployment, taken: &[bool], place: &SquadPlace) -> Option<usize> {
    deployment
        .squads
        .iter()
        .enumerate()
        .position(|(i, squad)| !taken[i] && squad.label == place.squad())
}
