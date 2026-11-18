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
use oc_root::uuids::SquadUuid;
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
    EnagedBattleMiddle,
    DeploySouthBeach,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_from(crate::args());
    logging::setup_logging()?;

    let mod_ = Mod::load(&PathBuf::from("mods/std1"), None)?;
    let map_ = PathBuf::from("maps/minidblue");
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

        Case::EnagedBattleMiddle => deployment::Deployments::from_files(
            &PathBuf::from("examples/deployment1a.yml"),
            &PathBuf::from("examples/deployment1b.yml"),
        )
        .unwrap(),
        Case::DeploySouthBeach => deployment::Deployments::from_files(
            &PathBuf::from("examples/deployment_allied1.yml"),
            &PathBuf::from("examples/deployment_axis1.yml"),
        )
        .unwrap()
        .with_spawns(
            vec![find_spawn(&spawns, "SouthBeach")],
            vec![find_spawn(&spawns, "Middle"), find_spawn(&spawns, "North")],
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
            Case::EnagedBattleMiddle => Supports::default(),
            Case::DeploySouthBeach => Supports::new(
                Support(vec![(SupportType::new(NAVAL_BOMBARD), 3)]),
                Support::default(),
            ),
        });

    snapshot.spawns = deployments
        .as_ref()
        .map(|d| d.spawns.iter().map(|s| s.name.clone()).collect());

    match args.case {
        Case::Empty => {}
        Case::EnagedBattleMiddle => {
            deploy_engaged_battle_middle(mod_, w, deployments, tiles, places, &mut snapshot)?;
        }
        Case::DeploySouthBeach => {
            deploy_south_beach(
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

fn deploy_engaged_battle_middle(
    mod_: Mod,
    w: WorldConfig,
    deployments: deployment::Deployments,
    tiles: Vec<oc_world::tile::Tile>,
    places: HashMap<PlaceName, WorldVec2>,
    snapshot: &mut Snapshot,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    // We use a hand made Placer to place squad manually for this case
    let placer = Placer::new(vec![])
        .with_assigned(vec![
            // uuids are these from examples/deployment1a.yml & examples/deployment1b.yml
            // places names are from examples/minidblue
            // A
            (
                "f9dbffe2-64ba-4a88-920b-ae883751493e"
                    .parse::<SquadUuid>()
                    .unwrap(),
                *places.get(&"63".into()).unwrap(),
            ),
            (
                "ca3db430-f7fa-4232-b2a1-2abb85499ae5"
                    .parse::<SquadUuid>()
                    .unwrap(),
                *places.get(&"50".into()).unwrap(),
            ),
            // B
            (
                "e1b6ec3d-7119-41d1-9f89-a6faf7da5bbb"
                    .parse::<SquadUuid>()
                    .unwrap(),
                *places.get(&"70".into()).unwrap(),
            ),
            (
                "9dbf54c7-3723-4fe3-a42b-44f3866c09aa"
                    .parse::<SquadUuid>()
                    .unwrap(),
                *places.get(&"53".into()).unwrap(),
            ),
        ])
        .with_orders(BySide::new(OrderPolicy::Hide, OrderPolicy::Defend));

    deployments
        // FIXME BS NOW: 1/ Début de partie ne doit pas impliquer mouvement de membres pour se place.
        //                  Le placer doit placer les individus a leur place directement.
        //               2/ Les soldats ne doivent pas tirer dès le début car hide. Là il y a des tirs.
        //                  Maybe individual n'ont pas encore reçu order (placer doit donner order)
        //                  Sinon c'est du tuning de behavior.
        // Sinon, créer les mécanique de la phase "deployment"
        .mobilize(&w, &mod_, snapshot, &placer, &tiles)
        .context("Generate snapshot from deployment files".to_string())?;

    snapshot.phase = BattlePhase::Fight;

    Ok(())
}

fn deploy_south_beach(
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
        Side::A => vec![
            // B
            (
                // Mg34
                "53f27839-4f3f-486e-bb47-420f382be19c"
                    .parse::<SquadUuid>()
                    .unwrap(),
                *places.get(&"50".into()).unwrap(),
            ),
            (
                // Mg34
                "03f99100-deff-4bd6-9b24-d3a4e1b3bbf6"
                    .parse::<SquadUuid>()
                    .unwrap(),
                *places.get(&"56".into()).unwrap(),
            ),
            (
                // Rifle
                "04f4eec9-fc48-4bf9-a2cd-4d072656e4a7"
                    .parse::<SquadUuid>()
                    .unwrap(),
                *places.get(&"42".into()).unwrap(),
            ),
            (
                // Rifle
                "578150d7-4cd9-448f-82cb-44c70e1da22b"
                    .parse::<SquadUuid>()
                    .unwrap(),
                *places.get(&"49".into()).unwrap(),
            ),
        ],
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
