#![allow(clippy::unwrap_used)]

use std::{
    path::PathBuf,
    sync::{Arc, mpsc::channel},
    time::Duration,
};

use crate::{
    logging,
    snapshot::SnapshotBuilder,
    utils::{Firing, Loaded, direction},
};
use oc_network::ToServer;
use oc_root::{side::Side, static_::StaticSource};
#[cfg(feature = "test")]
use oc_world_server::tracker::Tracker;
use oc_world_server::{bridge::Event, config::ServerConfig};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let world = PathBuf::from("maps/world1");
    let mod_ = PathBuf::from("mods/tests1");
    let Loaded {
        mod_: mod__,
        map,
        w,
        ..
    } = Loaded::new(&mod_, &world)?;
    let roots = oc_root::files::Roots::in_place(
        &PathBuf::from("mods"),
        &PathBuf::from("examples"),
        &PathBuf::from(".cache"),
    )?;
    roots.ensure_server_dirs()?;
    let static_ = StaticSource::Remote(32000);
    let snapshot = SnapshotBuilder::fight(map.clone(), vec![], vec![]).build(w.clone(), &mod__)?;
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

    let firing = Firing::weapon1(&mod__);

    let right = map.width().unwrap() as f32 * w.geo_pixels_per_tile() as f32;
    let bottom = map.height().unwrap() as f32 * w.geo_pixels_per_tile() as f32;

    let mut counter = 0;
    loop {
        if counter < 100_000 {
            for _ in 0..100 {
                let from = [0., 0., 500.];
                let directions = vec![direction(from, [right, bottom, 500.])];
                to_server_tx
                    .send(Event::Message(
                        (),
                        ToServer::ExplodeProjectile(firing.spawn(1, from, directions, Side::A)),
                    ))
                    .unwrap();
                counter += 1;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
