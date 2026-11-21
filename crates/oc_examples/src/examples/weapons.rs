#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use bevy::prelude::*;

use crate::{
    logging, run,
    snapshot::SnapshotBuilder,
    utils::{Loaded, armed_soldier, squad},
};
use clap::{Parser, ValueEnum};
use oc_individual::order::Order;
use oc_root::{WorldConfig, geo::WorldVec2, side::Side, utils::Frequency};
use oc_world::meta::Meta;

const MOD: &str = "mods/tests1";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    Stress,
    InaccuracyFastAndFar,
}

// FIXME BS NOW: ca rame plus le temps passe. Inspecter.
#[allow(unreachable_code)]
pub fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    #[cfg(not(feature = "debug"))]
    panic!("You must enable 'debug' feature for this example");

    let args = Args::parse_from(crate::args());
    let mod_ = PathBuf::from(MOD);
    let map = PathBuf::from("maps/world1");
    let loaded = Loaded::new(&mod_, &map)?;

    let w = match args.case {
        TestCase::InaccuracyFastAndFar | TestCase::Stress => {
            loaded.w.with_individual_tick(Frequency::new(10.0))
        }
    };

    let individuals = individuals(&w, &loaded.mod_, &loaded.meta, &args);
    let squads = squads(&individuals, &args);
    let snapshot =
        SnapshotBuilder::fight(loaded.map, individuals, squads).build(w, &loaded.mod_)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .snapshot(snapshot);

    example.build().run()?;

    Ok(())
}

/// Individual armed with "FullFast" weapon
fn individual(
    w: &WorldConfig,
    mod_: &oc_mod::Mod,
    position: [f32; 3],
) -> oc_individual::Individual {
    armed_soldier(w, mod_, Side::A, position, "FullFast")
}

fn individuals(
    w: &WorldConfig,
    mod_: &oc_mod::Mod,
    meta: &Meta,
    args: &Args,
) -> Vec<oc_individual::Individual> {
    match args.case {
        TestCase::InaccuracyFastAndFar => vec![
            individual(w, mod_, [52.5, 4450., 0.]),
            individual(
                w,
                mod_,
                [
                    497.,
                    4510.,
                    99. * meta.geo_meters_per_z * w.geo_pixels_per_meters(),
                ],
            ),
        ],
        TestCase::Stress => (0..10)
            .flat_map(|x| (0..10).map(move |y| [52.5 + 50. * x as f32, 4250. + 50. * y as f32, 0.]))
            .map(|position| individual(w, mod_, position))
            .collect(),
    }
}

fn squads(
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        TestCase::InaccuracyFastAndFar => vec![
            squad(
                individuals,
                &[0],
                vec![Order::Suppress(WorldVec2::new(255., 4100.))],
            ),
            squad(
                individuals,
                &[1],
                vec![Order::Suppress(WorldVec2::new(950., 3882.))],
            ),
        ],
        TestCase::Stress => (0..10)
            .flat_map(|x| (0..10).map(move |y| (x, y)))
            .enumerate()
            .map(|(i, (x, y))| {
                let target = WorldVec2::new(255. + 50. * x as f32, 4100. + 50. * y as f32);
                squad(individuals, &[i as u64], vec![Order::Suppress(target)])
            })
            .collect(),
    }
}
