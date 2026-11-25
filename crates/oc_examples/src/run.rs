use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc::channel},
    time::Duration,
};

use anyhow::Context;
use bevy::app::AppExit;
use bon::Builder;
use oc_network::ToServer;
use oc_root::side::Side;
use oc_root::{files::Roots, static_::StaticSource};
use oc_world::{load::WorldPath, meta::Meta};
#[cfg(feature = "test")]
use oc_world_server::tracker::Tracker;
use oc_world_server::{ai::AiConfig, config::ServerConfig, end::EndPolicy};

use crate::bridge;

#[cfg(feature = "test")]
type Result_ = Result<Tracker, anyhow::Error>;

#[cfg(not(feature = "test"))]
type Result_ = Result<(), anyhow::Error>;

type InstallFn = Box<dyn Fn(&mut bevy::app::App)>;

/// Server tracker, available to the gui app (tests can follow server events while running)
#[cfg(feature = "test")]
#[derive(bevy::prelude::Resource, Clone)]
pub struct TrackerRes(pub Tracker);

#[derive(Builder)]
pub struct Example {
    mod_: PathBuf,
    world: PathBuf,
    snapshot: PathBuf,
    install: Option<InstallFn>,
    #[builder(default)]
    test_app_exit_code: bool,
    #[builder(default = 10)]
    regions_width: u64,
    #[builder(default = 10)]
    regions_height: u64,
    region_width: Option<u64>,
    region_height: Option<u64>,
    #[builder(default = Side::A)]
    side: Side,
    #[builder(default)]
    ais: Vec<AiConfig>,
    #[builder(default)]
    see_all_enemies: bool,
    auto_fight: Option<Duration>,
}

impl Example {
    pub fn run(self) -> Result_ {
        // Server and gui share files: sources are used in place, derived files go under `.cache`
        let parent = |path: &PathBuf| -> anyhow::Result<PathBuf> {
            Ok(std::path::absolute(path)
                .context(format!("Absolute path of {}", path.display()))?
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default())
        };
        let roots = Roots::in_place(
            &parent(&self.mod_)?,
            &parent(&self.world)?,
            &PathBuf::from(".cache"),
        )
        .context("Build files roots")?;
        roots
            .ensure_server_dirs()
            .context(format!("Create dirs under {}", roots.derived.display()))?;

        #[cfg(feature = "test")]
        let tracker = Tracker::default();

        let (ready_tx, ready_rx) = channel::<std::result::Result<(), String>>();
        let (_, stop_rx) = channel::<()>();
        let (to_client_tx, to_client_rx) = channel();
        let (to_server_tx, to_server_rx) = channel();
        let _ = Meta::from_file(&self.world.meta())
            .context(format!("Read file {}", self.world.meta().display()))?;

        let map = oc_world::reader::MapReader::new(&self.world)
            .context(format!("Read map {}", self.world.display()))?;
        let map_width = map.width().context("Read map width".to_string())?;
        let map_height = map.height().context("Read map height".to_string())?;

        tracing::info!("Start server");

        let static_ = StaticSource::Local;
        let config = ServerConfig::builder()
            .world(self.world.clone())
            .mod_(self.mod_.clone())
            .roots(roots.clone())
            .static_(static_)
            .snapshot(self.snapshot.clone())
            .region_width(self.region_width.unwrap_or(map_width as u64))
            .region_height(self.region_height.unwrap_or(map_height as u64))
            .build();
        let state = oc_world_server::state::init::<()>(config.clone(), &map)?;
        let state = Arc::new(state);

        {
            #[cfg(feature = "test")]
            let tracker = tracker.clone();
            #[cfg(feature = "debug")]
            let side = self.side;
            let ais = self.ais.clone();
            std::thread::spawn(move || {
                oc_world_server::runner::Runner::new(
                    config,
                    state,
                    to_client_tx,
                    #[cfg(feature = "test")]
                    tracker,
                    #[cfg(feature = "debug")]
                    Some(((), side)),
                    EndPolicy::None,
                )
                .with_ais(ais)
                .run(to_server_rx, ready_tx, stop_rx);
            });
        }

        tracing::info!("Waiting server ...");
        let _ = ready_rx.recv().context("Wait server ready")?;

        let (client_tx2, server_rx2) = bridge::bridge(to_client_rx, to_server_tx);
        if let Some(delay) = self.auto_fight {
            auto_fight(client_tx2.clone(), delay);
        }

        tracing::info!("Start gui");
        let server_rx2 = Arc::new(Mutex::new(server_rx2));
        let connect = oc_battle_gui::config::Connect::Embedded(client_tx2, server_rx2);
        let config = oc_battle_gui::config::Config_::builder()
            .roots(roots)
            .auto_connect((connect, self.side))
            .regions_width(self.regions_width)
            .regions_height(self.regions_height)
            .see_all_enemies(self.see_all_enemies);
        let config = config.build();

        #[cfg(feature = "test")]
        let install: Option<InstallFn> = {
            let tracker = tracker.clone();
            let install = self.install;
            Some(Box::new(move |app: &mut bevy::app::App| {
                app.insert_resource(TrackerRes(tracker.clone()));
                if let Some(install) = &install {
                    install(app);
                }
            }))
        };
        #[cfg(not(feature = "test"))]
        let install = self.install;

        let app_exit = oc_battle_gui::run::run()
            .config(config)
            .maybe_install(install)
            .logging(false)
            .call();

        if self.test_app_exit_code
            && let AppExit::Error(code) = app_exit
        {
            let exit_code: u8 = code.into();
            std::process::exit(exit_code as i32);
        }

        #[cfg(feature = "test")]
        {
            Ok(tracker)
        }

        #[cfg(not(feature = "test"))]
        {
            Ok(())
        }
    }
}

/// Send fight request to server after `delay` (like the gui fight button)
fn auto_fight(server_tx: std::sync::mpsc::Sender<ToServer>, delay: Duration) {
    std::thread::spawn(move || {
        std::thread::sleep(delay);
        tracing::info!("Auto fight");
        let _ = server_tx.send(ToServer::Fight);
    });
}
