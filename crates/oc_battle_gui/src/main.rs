use std::{net::SocketAddr, path::PathBuf};

use clap::{Parser, Subcommand};
use oc_battle_gui::{
    config::{Config_, Connect},
    embedded::Embedded,
    run,
};
use oc_root::{files::Roots, side::Side};
use oc_world_server::args::ServerArgs;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub mode: Option<Mode>,
    /// Side to play when connect auto
    #[clap(long, value_enum, default_value = "a", global = true)]
    pub side: Side,
    #[clap(long, global = true)]
    pub see_all_enemies: bool,
    #[cfg(feature = "debug")]
    #[clap(long, global = true)]
    pub debug_auto_end: bool,
    /// With --debug-auto-end, kill given squad before ending (like `a:0` for first squad of side
    /// A). Can be repeated.
    #[cfg(feature = "debug")]
    #[clap(long, global = true, value_parser = oc_battle_gui::auto_end::parse_kill_squad)]
    pub debug_kill_squad: Vec<(Side, u32)>,
    /// With --debug-auto-end, give all flags to given side before ending
    #[cfg(feature = "debug")]
    #[clap(long, value_enum, global = true)]
    pub debug_own_flags: Option<Side>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Mode {
    /// Connect to a world server through network
    Network {
        /// World server address (like 127.0.0.1:6589)
        address: SocketAddr,
        /// Directory where files downloaded from server are stored (default: user data directory)
        #[clap(long)]
        data_dir: Option<PathBuf>,
    },
    /// Run a world server in this process
    Embedded(Box<ServerArgs>),
}

impl Args {
    fn config(self) -> anyhow::Result<Config_> {
        let (roots, connect) = match self.mode {
            None => (Roots::under(&Roots::default_data_dir())?, None),
            Some(Mode::Network { address, data_dir }) => {
                let data_dir = data_dir.unwrap_or_else(Roots::default_data_dir);
                (Roots::under(&data_dir)?, Some(Connect::Network(address)))
            }
            Some(Mode::Embedded(args)) => {
                let (roots, connect) = Embedded::builder()
                    .args(*args)
                    .side(self.side)
                    .build()
                    .start()?;
                (roots, Some(connect))
            }
        };

        Ok(Config_::builder()
            .maybe_auto_connect(connect.map(|c| (c, self.side)))
            .roots(roots)
            .see_all_enemies(self.see_all_enemies)
            .build())
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    #[cfg(feature = "debug")]
    let install = args.debug_auto_end.then(|| -> run::InstallFn {
        let plugin = oc_battle_gui::auto_end::AutoEndPlugin {
            kill_squads: args.debug_kill_squad.clone(),
            own_flags: args.debug_own_flags,
        };
        Box::new(move |app| {
            app.add_plugins(plugin.clone());
        })
    });
    #[cfg(not(feature = "debug"))]
    let install = None;

    let config = args.config()?;
    run::run()
        .config(config)
        .maybe_install(install)
        .logging(std::env::var("RUST_LOG").is_ok())
        .call();

    Ok(())
}
