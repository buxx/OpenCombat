#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
#[cfg(feature = "test")]
use std::sync::Mutex;
#[cfg(feature = "test")]
use std::sync::atomic::AtomicBool;
#[cfg(feature = "test")]
use std::sync::atomic::Ordering;
#[cfg(feature = "test")]
use std::time::Instant;

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use bevy::prelude::*;
use clap::{Parser, ValueEnum};
#[cfg(feature = "test")]
use oc_battle_gui::entity::individual::IndividualIndex;
use oc_battle_gui::ingame::GameConfigReceived;
use oc_battle_gui::ingame::lov::{Lov, UpdateLovFor};
use oc_battle_gui::world::InsertedTiles;
use oc_individual::order::Order;
#[cfg(feature = "test")]
use oc_physics::update::bevy::Position;
use oc_root::battle::BattlePhase;
use oc_root::geo::{WorldVec2, WorldVec3};
use oc_root::{Wcfg, side};
use oc_root::{WorldConfig, physics::Meters};
#[cfg(feature = "test")]
use oc_utils::d2::AlmostEqual;
use oc_world::meta::Meta;
use tests::individual::TestIndividual;
use tests::squad::TestSquad;

#[cfg(feature = "test")]
static SUCCESS: AtomicBool = AtomicBool::new(false);

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: Case,

    #[arg(long)]
    test: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Case {
    Visibilities,
    Climb,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_from(crate::args());
    logging::setup_logging()?;

    #[cfg(not(feature = "test"))]
    if args.test {
        panic!("Execute with --test need `test` feature enabled")
    }

    let mod_ = oc_mod::Mod::load(&PathBuf::from("mods/std1"), None)?;
    let meta = Meta::from_file(&PathBuf::from("maps/height/meta.toml"))?;
    let map_ = PathBuf::from("maps/height");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));

    let (individuals, squads) = match args.case {
        Case::Visibilities => (vec![], vec![]),
        Case::Climb => (
            vec![
                TestIndividual::builder()
                    .side(side::Side::A)
                    .position(WorldVec3::new(85., 50., 0.))
                    .build()
                    .make(&w),
            ],
            vec![
                TestSquad::builder()
                    .position(WorldVec2::new(85., 50.))
                    .members(vec![oc_individual::IndividualIndex(0)])
                    .orders(vec![Order::MoveFastTo(WorldVec2 { x: 10., y: 50. })])
                    .build()
                    .make(),
            ],
        ),
    };

    let snapshot = SnapshotBuilder::new(
        map,
        individuals,
        squads,
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .build(w, &mod_)?;

    let example = run::Example::builder()
        .world(PathBuf::from("maps/height"))
        .mod_(PathBuf::from("mods/std1"))
        .install(Box::new(setup))
        .snapshot(snapshot);

    example.build().run()?;

    #[cfg(feature = "test")]
    match SUCCESS.load(Ordering::Relaxed) {
        true => println!("✅ SUCCESS !"),
        false => return Err("❌ FAILED !".into()),
    }

    Ok(())
}

#[derive(Debug, Event)]
struct SetupLovs;

fn setup(app: &mut bevy::app::App) {
    let args = Args::parse_from(crate::args());

    match args.case {
        Case::Visibilities => {
            app.add_observer(on_game_config_received)
                .add_observer(on_inserted_tiles)
                .add_observer(on_setup_lovs);
        }
        Case::Climb => {}
    }

    #[cfg(feature = "test")]
    app.add_systems(Update, track);
}

fn on_game_config_received(
    _: On<GameConfigReceived>,
    mut commands: Commands,
    world: Res<oc_battle_gui::world::World>,
) {
    if !world.tiles.is_empty() {
        commands.trigger(SetupLovs);
    }
}

// TODO: This is a tricky way to know when on_update_lov_for is ready to receive
// find a way to trigger when all these config are received, or send all in same time
fn on_inserted_tiles(_: On<InsertedTiles>, w: Res<Wcfg>, mut commands: Commands) {
    if w.0.is_some() {
        commands.trigger(SetupLovs);
    }
}

fn on_setup_lovs(_: On<SetupLovs>, _: Res<Wcfg>, mut commands: Commands) {
    let lovs = vec![
        // dans la plaine, aucun obstacle
        (Vec3::new(65., 50., 12.), Vec2::new(183., 50.)),
        // plaine, mur de brique
        (Vec3::new(71., 124., 7.5), Vec2::new(181., 123.)),
        // plaine grande montagne
        (Vec3::new(70., 172., 7.), Vec2::new(183., 172.)),
        // plaine petite montagne
        (Vec3::new(71., 227., 7.), Vec2::new(185., 227.)),
        // plaine herbe hautes
        (Vec3::new(65., 77., 12.), Vec2::new(180., 76.)),
        // plaine sous bois
        (Vec3::new(65., 22., 12.), Vec2::new(184., 22.)),
        // semi colline par dessus moyenne montagne
        (Vec3::new(29., 242., 52.), Vec2::new(216., 241.)),
        // haute colline par dessus moyenne montagne
        (Vec3::new(11., 207., 57.), Vec2::new(241., 207.)),
        // semi colline haute montagne
        (Vec3::new(40., 188., 37.), Vec2::new(214., 186.)),
        // haute colline par dessus haute montagne
        (Vec3::new(3., 157., 57.), Vec2::new(246., 157.)),
        // semi colline par dessus mur brique
        (Vec3::new(30., 131., 47.), Vec2::new(222., 131.)),
        // semi colline par dessus herbe haute
        (Vec3::new(13., 71., 57.), Vec2::new(241., 71.)),
        // semi colline par dessus sous bois
        (Vec3::new(31., 30., 47.), Vec2::new(233., 30.)),
    ];

    for (start, end) in lovs {
        let lov = Lov {
            start: WorldVec3::new(start.x, start.y, start.z),
            stop: WorldVec3::new(start.x, start.y, start.z),
            stop_plus_z: Meters(0.),
            sections: vec![],
            max: None,
        };
        let entity = commands.spawn(lov);
        let update = UpdateLovFor(entity.id(), WorldVec2::new(end.x, end.y));
        commands.trigger(update);
    }
}

#[cfg(feature = "test")]
fn track(individuals: Query<(&Position, &IndividualIndex)>, mut commands: Commands) {
    static IO_REACH_POSITION: Mutex<Option<Instant>> = Mutex::new(None);

    let i0_position = individuals
        .iter()
        .find(|(_, i)| i.0 == oc_individual::IndividualIndex(0))
        .map(|(p, _)| p.0);

    if i0_position
        .map(|p| {
            p.almost_equal(
                WorldVec3 {
                    x: 10.,
                    y: 50.,
                    z: 45.5,
                },
                10., // TODO: that a large "almost", but we need a more precise path step mechanism
            )
        })
        .unwrap_or_default()
        && IO_REACH_POSITION.lock().unwrap().is_none()
    {
        *IO_REACH_POSITION.lock().unwrap() = Some(Instant::now());
    }

    if IO_REACH_POSITION
        .lock()
        .unwrap()
        .map(|i| i.elapsed().as_secs() >= 1)
        .unwrap_or_default()
    {
        SUCCESS.store(true, Ordering::Relaxed);
        commands.write_message(bevy::app::AppExit::from_code(0));
    }
}
