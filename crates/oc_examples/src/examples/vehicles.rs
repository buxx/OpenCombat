#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use clap::{Parser, ValueEnum};
use oc_geo::tile::WorldTileIndex;
use oc_individual::IndividualIndex;
use oc_mod::{Mod, vehicle::CrewRole};
use oc_root::{
    WcfgFrom, WorldConfig, battle::BattlePhase, geo::WorldVec2, physics::Meters, side::Side,
};
use oc_utils::{d2::Direction, index::AtIndex};
use oc_vehicle::{Vehicle, VehicleIndex};
use oc_world::{meta::Meta, tile::Tile};
use tests::{individual::TestIndividual, squad::TestSquad};

const MOD: &str = "mods/tests1";
const MAP: &str = "maps/meadow1";
const TANK: &str = "Tank1";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    TankMoveForward,
    DrawTanks,
}

struct TankSetup {
    side: Side,
    position: WorldVec2,
    chassis: Direction,
    turret: Option<Direction>,
    crew: Vec<CrewRole>,
}

impl TankSetup {
    fn new(side: Side, position: [f32; 2], chassis: Direction, turret: Option<Direction>) -> Self {
        Self {
            side,
            position: WorldVec2::new(position[0], position[1]),
            chassis,
            turret,
            crew: vec![CrewRole::Driver],
        }
    }

    fn with_crew(mut self, crew: Vec<CrewRole>) -> Self {
        self.crew = crew;
        self
    }
}

// FIXME BS NOW : tests and code to implement:
// - Tank move (individual behavior => vehicle physics)
// - Individual in vahicle protected against bullets, shrapnels, suppress, but not against stunned
// - projectile collision with vehicle (all chassis surface, when oriented too)
// - Tank destruction (penetration) and weak parts
// - Tank fire/reload/ammunitions
// - Tank chassis visible (when crew hidden position)
pub fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    let args = Args::parse_from(crate::args());
    let mod_ = PathBuf::from(MOD);
    let mod__ = oc_mod::Mod::load(&mod_, None)?;
    let map = PathBuf::from(MAP);
    let meta = Meta::from_file(&map.join("meta.toml"))?;
    let map_ = oc_world::reader::MapReader::new(&map);
    let map_ = map_.context(format!("Read map_ {}", map.display()))?;
    let w = WorldConfig::new(map_.width().unwrap() as u64, map_.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    let tiles = map_.tiles(&w, &mod__).unwrap();

    let tanks = tanks(&args);
    let individuals = individuals(&w, &tiles, &tanks, &args);
    let squads = squads(&w, &tiles, &individuals, &args);
    let vehicles = vehicles(&w, &tiles, &mod__, &tanks);
    let snapshot = SnapshotBuilder::new(
        map_,
        individuals,
        squads,
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .with_vehicles(vehicles)
    .build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .snapshot(snapshot);

    example.build().run()?;

    Ok(())
}

fn tanks(args: &Args) -> Vec<TankSetup> {
    let diagonal = Direction::new(1., 1.);
    match args.case {
        // TODO: tank with move forward order
        TestCase::TankMoveForward => vec![],
        TestCase::DrawTanks => vec![
            // Turret aligned with chassis, full crew
            TankSetup::new(
                Side::A,
                [150., 250.],
                Direction::NORTH,
                Some(Direction::NORTH),
            )
            .with_crew(vec![
                CrewRole::Commander,
                CrewRole::Gunner,
                CrewRole::Driver,
                CrewRole::Operator,
            ]),
            // Turret rotated relatively to chassis
            TankSetup::new(
                Side::A,
                [200., 250.],
                Direction::EST,
                Some(Direction::NORTH),
            ),
            TankSetup::new(Side::A, [250., 250.], diagonal, Some(Direction::WEST)),
            // Without turret
            TankSetup::new(Side::A, [300., 250.], Direction::SOUTH, None),
            // Enemy: visible when its crew is visible
            TankSetup::new(Side::B, [225., 190.], Direction::SOUTH, Some(diagonal)),
        ],
    }
}

fn z(w: &WorldConfig, tiles: &[Tile], position: WorldVec2) -> f32 {
    tiles.at(WorldTileIndex::from_(position, w)).z_pixels(w)
}

fn individuals(
    w: &WorldConfig,
    tiles: &[Tile],
    tanks: &[TankSetup],
    args: &Args,
) -> Vec<oc_individual::Individual> {
    match args.case {
        // TODO: tank with move forward order
        TestCase::TankMoveForward => vec![],
        TestCase::DrawTanks => crews(w, tiles, tanks),
    }
}

fn crews(w: &WorldConfig, tiles: &[Tile], tanks: &[TankSetup]) -> Vec<oc_individual::Individual> {
    let mut individuals = vec![];

    for (i, tank) in tanks.iter().enumerate() {
        let position = tank.position.extend(z(w, tiles, tank.position));
        for role in &tank.crew {
            let individual = TestIndividual::builder()
                .side(tank.side)
                .position(position)
                .build()
                .make(w)
                .with_embarked(Some((VehicleIndex(i as u64), *role)));
            individuals.push(individual);
        }
    }

    individuals
}

fn squads(
    _w: &WorldConfig,
    _tiles: &[Tile],
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        // TODO: tank with move forward order
        TestCase::TankMoveForward => vec![],
        TestCase::DrawTanks => crew_squads(individuals),
    }
}

fn crew_squads(individuals: &[oc_individual::Individual]) -> Vec<oc_individual::squad::Squad> {
    let mut squads: Vec<(Option<VehicleIndex>, Vec<IndividualIndex>)> = vec![];

    for (i, individual) in individuals.iter().enumerate() {
        let vehicle = individual.embarked.map(|(vehicle, _)| vehicle);
        match squads.last_mut() {
            Some((vehicle_, members)) if vehicle.is_some() && *vehicle_ == vehicle => {
                members.push(IndividualIndex(i as u64));
            }
            _ => squads.push((vehicle, vec![IndividualIndex(i as u64)])),
        }
    }

    squads
        .into_iter()
        .map(|(_, members)| {
            let leader = &individuals[members[0].0 as usize];
            TestSquad::builder()
                .side(leader.side)
                .position(leader.position.into())
                .members(members)
                .build()
                .make()
        })
        .collect()
}

fn vehicles(w: &WorldConfig, tiles: &[Tile], mod_: &Mod, tanks: &[TankSetup]) -> Vec<Vehicle> {
    let type_ = mod_.vehicle_from_name(TANK).unwrap().index();

    tanks
        .iter()
        .map(|tank| {
            Vehicle::new(
                tank.side,
                type_,
                oc_mod::illustration::IllustrationIndex(0),
                tank.position.extend(z(w, tiles, tank.position)),
                vec![],
                tank.chassis,
                tank.turret,
            )
        })
        .collect()
}
