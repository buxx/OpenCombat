#![allow(clippy::unwrap_used)]

use std::{
    path::PathBuf,
    sync::{Arc, mpsc::channel},
    time::Duration,
};

use crate::{logging, snapshot::SnapshotBuilder};
use anyhow::Context as ContextExt;
use bevy::prelude::*;
use oc_mod::Mod;
use oc_network::ToServer;
use oc_projectile::spawn::SpawnProjectiles;
use oc_root::{
    WorldConfig, battle::BattlePhase, geo::WorldVec3, physics::Meters, side::Side,
    static_::StaticSource,
};
use oc_world::meta::Meta;
#[cfg(feature = "test")]
use oc_world_server::tracker::Tracker;
use oc_world_server::{bridge::Event, config::ServerConfig};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let world = PathBuf::from("maps/world1");
    let mod_ = PathBuf::from("mods/tests1");
    let mod__ = Mod::load(&mod_, None)?;
    let roots = oc_root::files::Roots::in_place(
        &PathBuf::from("mods"),
        &PathBuf::from("examples"),
        &PathBuf::from(".cache"),
    )?;
    roots.ensure_server_dirs()?;
    let static_ = StaticSource::Remote(32000);
    let meta = Meta::from_file(&PathBuf::from("maps/world1/meta.toml"))?;
    let map_ = PathBuf::from("maps/world1");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    let snapshot = SnapshotBuilder::new(
        map.clone(),
        vec![],
        vec![],
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .build(w.clone(), &mod__)?;
    let config = ServerConfig::builder()
        .world(world.clone())
        .mod_(mod_.clone())
        .roots(roots)
        .static_(static_)
        .snapshot(snapshot.clone())
        .region_width(100)
        .region_height(100)
        .build();
    let state = oc_world_server::state::init::<()>(config.clone(), &map)?;
    let state = Arc::new(state);

    let (ready_tx, ready_rx) = channel::<std::result::Result<(), String>>();
    let (_, stop_rx) = channel::<()>();
    let (to_client_tx, _) = channel();
    let (to_server_tx, to_server_rx) = channel();

    #[cfg(feature = "test")]
    let tracker = Tracker::default();

    std::thread::spawn(move || {
        oc_world_server::runner::Runner::new(
            config,
            state,
            to_client_tx,
            #[cfg(feature = "test")]
            tracker,
            #[cfg(feature = "debug")]
            None,
            oc_world_server::end::EndPolicy::None,
        )
        .run(to_server_rx, ready_tx, stop_rx);
    });

    let _ = ready_rx.recv().unwrap();

    let weapon = mod__
        .weapons
        .iter()
        .find(|w| w.name() == "Weapon1")
        .unwrap();
    let ammunition = weapon
        .ammunitions()
        .iter()
        .find(|a| a.name() == "Ammo1")
        .unwrap();
    let shot = weapon
        .shots()
        .iter()
        .find(|s| s.name() == "Single")
        .unwrap();

    let right = map.width().unwrap() as f32 * w.geo_pixels_per_tile() as f32;
    let bottom = map.height().unwrap() as f32 * w.geo_pixels_per_tile() as f32;

    let mut counter = 0;
    loop {
        if counter < 100_000 {
            for _ in 0..100 {
                let directions: Vec<WorldVec3> = vec![
                    (WorldVec3::new(right, bottom, 500.) - WorldVec3::new(0., 0., 500.))
                        .normalize_or_zero(),
                ];
                to_server_tx
                    .send(Event::Message(
                        (),
                        ToServer::ExplodeProjectile(SpawnProjectiles::new(
                            weapon.index(),
                            ammunition.index(),
                            shot.index(),
                            1,
                            [0., 0., 500.].into(),
                            directions,
                            Side::A,
                            Duration::ZERO,
                            None,
                        )),
                    ))
                    .unwrap();
                counter += 1;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
