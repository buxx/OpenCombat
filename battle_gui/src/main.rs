use std::error::Error;
use std::fmt::Display;
use std::path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use battle_core::config::GuiConfig;
use battle_core::config::ServerConfig;
use battle_core::config::DEFAULT_SERVER_PUB_ADDRESS;
use battle_core::config::DEFAULT_SERVER_REP_ADDRESS;
use battle_core::game::Side;
use battle_core::map::reader::MapReader;
use battle_core::map::reader::MapReaderError;
use battle_core::message::InputMessage;
use battle_core::network::client::Client;
use battle_core::network::error::NetworkError;
use battle_core::state::battle::BattleState;
use crossbeam_channel::unbounded;
use crossbeam_channel::SendError;
use ggez::conf::WindowMode;
use ggez::event;
use ggez::GameError;
use oc_core::utils::SpawnZoneName;
use server::EmbeddedServer;

mod audio;
mod debug;
mod engine;
mod graphics;
mod physics;
mod server;
mod ui;
mod utils;

use server::EmbeddedServerError;
use structopt::StructOpt;

pub const RESOURCE_PATH: &'static str = "resources";

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(long = "--embedded-server")]
    embedded_server: bool,

    #[structopt(long = "--server-rep-address", default_value = DEFAULT_SERVER_REP_ADDRESS)]
    server_rep_address: String,

    #[structopt(long = "--server-bind-address", default_value = DEFAULT_SERVER_PUB_ADDRESS)]
    server_pub_address: String,

    #[structopt(long = "side")]
    side: Side,

    #[structopt(long = "profile")]
    profile: bool,

    #[structopt(long = "--profile-address", default_value = "0.0.0.0:8585")]
    profile_address: String,

    #[structopt(long = "side-a-control")]
    a_control: Vec<SpawnZoneName>,

    #[structopt(long = "side-b-control")]
    b_control: Vec<SpawnZoneName>,
}

fn main() -> Result<(), GuiError> {
    let opt = Opt::from_args();
    let map_name = "map1";
    let situation_name = "hardcoded";
    let resources = PathBuf::from("./resources");
    let sync_required = Arc::new(AtomicBool::new(true));
    let stop_required = Arc::new(AtomicBool::new(false));

    // Profiling server
    // NOTE : We must keep server object to avoid its destruction
    let _puffin_server = if opt.profile {
        let puffin_server = puffin_http::Server::new(&opt.profile_address).unwrap();
        puffin::set_scopes_on(true);
        Some(puffin_server)
    } else {
        None
    };

    let (input_sender, output_receiver) = if opt.embedded_server {
        let (input_sender, input_receiver) = unbounded();
        let (output_sender, output_receiver) = unbounded();

        EmbeddedServer::new(
            &resources,
            input_receiver,
            output_sender,
            stop_required.clone(),
        )
        .map_name(map_name)
        .situation_name(situation_name)
        .server_rep_address(&opt.server_rep_address)
        .server_pub_address(&opt.server_pub_address)
        .start()?;

        (input_sender, output_receiver)
    } else {
        let (input_sender, input_receiver) = unbounded();
        let (output_sender, output_receiver) = unbounded();

        Client::new(
            opt.server_rep_address.clone(),
            opt.server_pub_address.clone(),
            input_sender.clone(),
            input_receiver,
            output_sender,
            output_receiver.clone(),
            sync_required.clone(),
        )
        .connect()?;

        (input_sender, output_receiver)
    };

    input_sender.send(vec![InputMessage::RequireCompleteSync])?;

    let context_builder = ggez::ContextBuilder::new("Open Combat", "Bastien Sevajol")
        .add_resource_path(path::PathBuf::from(format!("./{}", RESOURCE_PATH)))
        .window_mode(
            WindowMode::default()
                .dimensions(1024., 768.)
                .resizable(true),
        );
    let (mut context, event_loop) = context_builder.build()?;

    // TODO : If remote server, download map before read it
    let map = MapReader::new(map_name, &resources)?.build()?;
    let config = GuiConfig::new();
    let server_config = ServerConfig::new();
    let graphics = graphics::Graphics::new(&mut context, &map, &server_config)?;
    let battle_state = BattleState::empty(&map);
    let engine = engine::Engine::new(
        &mut context,
        &opt.side,
        config,
        server_config,
        input_sender,
        output_receiver,
        graphics,
        battle_state,
        sync_required,
        stop_required.clone(),
        opt.a_control.clone(),
        opt.b_control.clone(),
    )?;

    // FIXME BS NOW : Closing GUI don't close thread correctly and keep process running
    println!("Start Gui");
    event::run(context, event_loop, engine)
}

#[derive(Debug)]
enum GuiError {
    MapReader(MapReaderError),
    RunGame(GameError),
    SendInput(SendError<Vec<InputMessage>>),
    Network(NetworkError),
    EmbeddedServer(EmbeddedServerError),
}

impl Error for GuiError {}

impl From<MapReaderError> for GuiError {
    fn from(error: MapReaderError) -> Self {
        Self::MapReader(error)
    }
}

impl From<GameError> for GuiError {
    fn from(error: GameError) -> Self {
        Self::RunGame(error)
    }
}

impl From<SendError<Vec<InputMessage>>> for GuiError {
    fn from(error: SendError<Vec<InputMessage>>) -> Self {
        Self::SendInput(error)
    }
}

impl From<NetworkError> for GuiError {
    fn from(error: NetworkError) -> Self {
        Self::Network(error)
    }
}

impl From<EmbeddedServerError> for GuiError {
    fn from(error: EmbeddedServerError) -> Self {
        Self::EmbeddedServer(error)
    }
}

impl Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiError::MapReader(error) => {
                f.write_str(&format!("Error during map load : {}", error))
            }
            GuiError::RunGame(error) => f.write_str(&format!("Running error : {}", error)),
            GuiError::SendInput(error) => {
                f.write_str(&format!("Error during input send : {}", error))
            }
            GuiError::Network(error) => f.write_str(&format!("Network error : {}", error)),
            GuiError::EmbeddedServer(error) => {
                f.write_str(&format!("Embedded server error : {}", error))
            }
        }
    }
}
