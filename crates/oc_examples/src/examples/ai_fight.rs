//! AI versus AI battle: both sides are played by the computer, gui only observes (use
//! `--see-all-enemies` to display all individuals). Spawn zones, supports and initial orders come from an existing
//! battle scenario of the chosen map.
//!
//! AIs deploy their squads during deployment phase: start the fight from the gui when ready, or
//! use `--auto-fight [SECONDS]` to start it automatically (after 5 seconds by default).
use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use clap::{Parser, ValueEnum};
use oc_ai::AiKind;
use oc_battle::scenario::Scenario;
use oc_deployment::deployment::place::OrderPolicy;
use oc_root::format::RkyvFile;
use oc_root::side::{BySide, Side};
use oc_world::support::Supports;
use oc_world_server::{
    ai::AiConfig,
    generate::{Generate, InitialOrder},
};

use crate::{logging, run};

const MOD: &str = "mods/std1";
/// Deployment (squads) files of sides A and B
const DEPLOYMENT_A: &str = "examples/deployment_allied1.yml";
const DEPLOYMENT_B: &str = "examples/deployment_axis1.yml";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Map (and its battle scenario) to fight on
    #[arg(long, value_enum, default_value = "minidblue")]
    map: Map,

    /// Behavior of side A AI (default: scenario one)
    #[arg(long, value_enum)]
    a_ai: Option<AiKind>,

    /// Behavior of side B AI (default: scenario one)
    #[arg(long, value_enum)]
    b_ai: Option<AiKind>,

    /// Side of the observer gui
    #[arg(long, value_enum, default_value = "a")]
    side: Side,

    /// Gui displays all opposite side individuals (not only the ones visible by its side)
    #[arg(long)]
    see_all_enemies: bool,

    /// Start fight automatically after given seconds (let AIs deploy their squads first)
    #[arg(long, value_name = "SECONDS", num_args = 0..=1, default_missing_value = "5")]
    auto_fight: Option<u64>,

    /// Fight duration (game time seconds), default is 15 minutes
    #[arg(long, value_name = "SECONDS")]
    duration: Option<u64>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Map {
    Demo2,
    Minidblue,
}

impl Map {
    fn world(self) -> PathBuf {
        PathBuf::from(match self {
            Map::Demo2 => "maps/demo2",
            Map::Minidblue => "maps/minidblue",
        })
    }

    fn scenario(self) -> PathBuf {
        // TODO: copy them in examples if changes in breaking way for the example
        PathBuf::from(match self {
            Map::Demo2 => "battles/Orchard_battle/Allied_push.ron",
            Map::Minidblue => "battles/Mini_D_day/Landing.ron",
        })
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_from(crate::args());
    logging::setup_logging()?;

    let world = args.map.world();
    let scenario_path = args.map.scenario();
    let scenario = Scenario::from_ron_file(&scenario_path)
        .context(format!("Read scenario {}", scenario_path.display()))?;
    let sides = scenario.sides();

    let kinds = BySide::new(args.a_ai, args.b_ai);
    let ais: Vec<AiConfig> = [Side::A, Side::B]
        .into_iter()
        .map(|side| AiConfig {
            side,
            kind: kinds[side].unwrap_or(sides[side].ai),
            placed: vec![],
        })
        .collect();
    for ai in &ais {
        tracing::info!("Side {:?} played by {} AI", ai.side, ai.kind.label());
    }

    let spawns = |side: Side| -> Vec<String> {
        sides[side]
            .start_spawn_zones
            .iter()
            .map(|z| z.0.clone())
            .collect()
    };
    let order = |side: Side| sides[side].order.map(initial_order);
    let generate = Generate::builder()
        .deployment_a(PathBuf::from(DEPLOYMENT_A))
        .deployment_b(PathBuf::from(DEPLOYMENT_B))
        .spawns_a(spawns(Side::A))
        .spawns_b(spawns(Side::B))
        .supports(Supports::new(
            sides[Side::A].supports.clone(),
            sides[Side::B].supports.clone(),
        ))
        .maybe_order_a(order(Side::A))
        .maybe_order_b(order(Side::B))
        .build();

    let map = oc_world::reader::MapReader::new(&world)
        .context(format!("Read map {}", world.display()))?;
    let mut snapshot = generate
        .snapshot(&PathBuf::from(MOD), &world, &map, None)
        .context("Generate snapshot")?;
    if let Some(duration) = args.duration {
        snapshot.duration = Duration::from_secs(duration);
    }
    let (_, snapshot_path) = tempfile::NamedTempFile::new()?.keep()?;
    snapshot
        .save(&snapshot_path)
        .context(format!("Save snapshot to {}", snapshot_path.display()))?;

    run::Example::builder()
        .world(world)
        .mod_(PathBuf::from(MOD))
        .snapshot(snapshot_path)
        .side(args.side)
        .ais(ais)
        .see_all_enemies(args.see_all_enemies)
        .maybe_auto_fight(args.auto_fight.map(Duration::from_secs))
        .build()
        .run()?;

    Ok(())
}

fn initial_order(policy: OrderPolicy) -> InitialOrder {
    match policy {
        OrderPolicy::None => InitialOrder::None,
        OrderPolicy::Hide => InitialOrder::Hide,
        OrderPolicy::Defend => InitialOrder::Defend,
    }
}
