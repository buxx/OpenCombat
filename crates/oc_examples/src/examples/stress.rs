#![allow(clippy::unwrap_used)]
use std::path::PathBuf;

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use bevy::prelude::*;
use clap::Parser;
use oc_battle_gui::{ingame::GameConfigReceived, network::output::ToServerEvent};
use oc_individual::{Individual, IndividualIndex, order::Order, squad::Squad};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_root::{
    WorldConfig,
    battle::BattlePhase,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    side::Side,
};
use oc_world::{meta::Meta, support::SupportType};
use tests::{
    individual::TestIndividual,
    squad::TestSquad,
    weapons::{TestWeapon, TestWeapons},
};
// WARNING: AI generated module

/// Support declared by mods std1 and tests1
const NAVAL_BOMBARD: &str = "Naval bombard";
const MOD: &str = "mods/std1";
const MAP: &str = "maps/world1";
const INDIVIDUALS_BY_SQUAD: usize = 5;
/// Distance between individuals of a same squad (pixels)
const MEMBERS_GAP: f32 = 6.;
const MARGIN: f32 = 30.;
/// Machine gun given to every individual, to stress projectiles physics
const WEAPON: &str = "Mg34";
/// Spare magazines by individual
const SPARE_MAGAZINES: usize = 10;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Squads count by side
    #[arg(long, default_value_t = 200)]
    squads: usize,
}

/// Squads grid (columns, rows) of one side, cells following the world aspect ratio
fn grid(w: &WorldConfig, squads: usize) -> (usize, usize) {
    let width = w.world_width_pixels() as f32 - 2. * MARGIN;
    let height = w.world_height_pixels() as f32 - 2. * MARGIN;
    let columns = ((squads as f32 * width / height).sqrt().ceil() as usize).clamp(1, squads.max(1));
    (columns, squads.div_ceil(columns))
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_from(crate::args());
    logging::setup_logging()?;

    #[cfg(feature = "debug")]
    WorldConfig::set_immortality(true);
    #[cfg(not(feature = "debug"))]
    tracing::warn!("`debug` feature disabled: no immortality, only side A requests bombardments");

    let mod_path = PathBuf::from(MOD);
    let mod_ = Mod::load(&mod_path, None)?;
    let meta = Meta::from_file(&PathBuf::from(MAP).join("meta.toml"))?;
    let map = oc_world::reader::MapReader::new(&PathBuf::from(MAP));
    let map = map.context(format!("Read map {MAP}"))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));

    let (individuals, squads) = battalions(&w, &mod_, args.squads);
    let snapshot = SnapshotBuilder::new(
        map,
        individuals,
        squads,
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .build(w.clone(), &mod_)?;

    let bombard = Bombard::new(&w, &mod_);
    let example = run::Example::builder()
        .world(PathBuf::from(MAP))
        .mod_(mod_path)
        .install(Box::new(move |app| install(app, bombard.clone())))
        .snapshot(snapshot);
    example.build().run()?;

    Ok(())
}

fn squad_position(w: &WorldConfig, side: Side, j: usize, squads: usize) -> WorldVec2 {
    let width = w.world_width_pixels() as f32 - 2. * MARGIN;
    let height = w.world_height_pixels() as f32 - 2. * MARGIN;
    let (columns, rows) = grid(w, squads);
    let (cell_w, cell_h) = (width / columns as f32, height / rows as f32);
    let shift = match side {
        Side::A => 0.25,
        Side::B => 0.75,
    };
    let (column, row) = (j % columns, j / columns);
    WorldVec2::new(
        MARGIN + (column as f32 + shift) * cell_w,
        MARGIN + (row as f32 + shift) * cell_h,
    )
}

fn battalions(w: &WorldConfig, mod_: &Mod, squads_by_side: usize) -> (Vec<Individual>, Vec<Squad>) {
    let (width, height) = (
        w.world_width_pixels() as f32,
        w.world_height_pixels() as f32,
    );
    let spare_magazines = vec![
        mod_.weapons
            .iter()
            .find(|w| w.name() == WEAPON)
            .unwrap()
            .magazines()
            .first()
            .unwrap()
            .index();
        SPARE_MAGAZINES
    ];
    let mut individuals = vec![];
    let mut squads = vec![];

    for side in [Side::A, Side::B] {
        for j in 0..squads_by_side {
            let position = squad_position(w, side, j, squads_by_side);
            let first = individuals.len();
            for k in 0..INDIVIDUALS_BY_SQUAD {
                individuals.push(
                    TestIndividual::builder()
                        .side(side)
                        .position(WorldVec3::new(
                            position.x + k as f32 * MEMBERS_GAP,
                            position.y,
                            0.,
                        ))
                        .weapons(
                            TestWeapons::builder()
                                .primary(TestWeapon::filled(mod_, WEAPON).make())
                                .build()
                                .make(),
                        )
                        .magazines(spare_magazines.clone())
                        .build()
                        .make(w),
                );
            }

            // Odd squads move to the opposite position on the map
            let orders = match j % 2 {
                1 => vec![Order::MoveTo(WorldVec2::new(
                    width - position.x,
                    height - position.y,
                ))],
                _ => vec![],
            };
            squads.push(
                TestSquad::builder()
                    .side(side)
                    .position(position)
                    .members(
                        (first..first + INDIVIDUALS_BY_SQUAD)
                            .map(IndividualIndex::from)
                            .collect(),
                    )
                    .orders(orders)
                    .build()
                    .make(),
            );
        }
    }

    (individuals, squads)
}

#[derive(Resource, Clone)]
struct Bombard {
    width: f32,
    height: f32,
    period: f32,
    next: [f32; 2],
    started: bool,
    rng: u32,
}

impl Bombard {
    fn new(w: &WorldConfig, mod_: &Mod) -> Self {
        let support = mod_
            .support(NAVAL_BOMBARD)
            .expect("Mod must declare naval bombard support");
        let shells = support.shells_by_serie;
        let serie = support.shell_interval() * shells + support.serie_interval();
        // Last explosion of the last shell of the last serie
        let last = support.prepare()
            + serie * support.series.saturating_sub(1)
            + support.shell_interval() * shells.saturating_sub(1);

        Self {
            width: w.world_width_pixels() as f32,
            height: w.world_height_pixels() as f32,
            period: last.as_secs_f32() / w.speed(),
            next: [0.; 2],
            started: false,
            rng: 0x9E37_79B9,
        }
    }

    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / (1u32 << 24) as f32
    }
}

fn install(app: &mut App, bombard: Bombard) {
    app.insert_resource(bombard)
        .add_observer(|_: On<GameConfigReceived>, mut bombard: ResMut<Bombard>| {
            tracing::info!("Game config received, start bombardments");
            bombard.started = true;
        })
        .add_systems(Update, request_bombards);
}

fn request_bombards(mut bombard: ResMut<Bombard>, time: Res<Time<Real>>, mut commands: Commands) {
    if !bombard.started {
        return;
    }

    let now = time.elapsed_secs();
    for (i, side) in [Side::A, Side::B].into_iter().enumerate() {
        if now < bombard.next[i] {
            continue;
        }
        bombard.next[i] = now + bombard.period;

        let position = WorldVec2::new(
            bombard.random() * bombard.width,
            bombard.random() * bombard.height,
        );
        #[cfg(not(feature = "debug"))]
        if side == Side::B {
            continue;
        }
        tracing::info!(?side, ?position, "Request naval bombard");
        #[cfg(feature = "debug")]
        commands.trigger(ToServerEvent(ToServer::DebugRequestSupport(
            side,
            SupportType::new(NAVAL_BOMBARD),
            position,
        )));
        #[cfg(not(feature = "debug"))]
        commands.trigger(ToServerEvent(ToServer::RequestSupport(
            SupportType::new(NAVAL_BOMBARD),
            position,
        )));
    }
}
