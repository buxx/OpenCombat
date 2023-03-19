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
use battle_core::state::battle::message::BattleStateMessage;
use battle_core::state::battle::BattleState;
use crossbeam_channel::unbounded;
use crossbeam_channel::SendError;
use ggez::conf::WindowMode;
use ggez::event;
use ggez::GameError;
use oc_core::resources::Resources;
use oc_core::resources::ResourcesError;
use oc_core::spawn::SpawnZoneName;
use server::EmbeddedServer;
use thiserror::Error;

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
    let sync_required = Arc::new(AtomicBool::new(true));
    let stop_required = Arc::new(AtomicBool::new(false));
    let resources = Resources::new()?.ensure()?;

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
            &resources.lib(),
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

    let ready_message = if &opt.side == &Side::A {
        InputMessage::BattleState(BattleStateMessage::SetAConnected(true))
    } else {
        InputMessage::BattleState(BattleStateMessage::SetBConnected(true))
    };
    input_sender.send(vec![InputMessage::RequireCompleteSync, ready_message])?;

    let mut context_builder = ggez::ContextBuilder::new("Open Combat", "Bastien Sevajol")
        .window_mode(
            WindowMode::default()
                .dimensions(1024., 768.)
                .resizable(true),
        );
    for resource_path in resources.resources_paths_abs() {
        context_builder = context_builder.add_resource_path(resource_path);
    }
    let (mut context, event_loop) = context_builder.build()?;

    // TODO : If remote server, download map before read it
    let map = MapReader::new(map_name, &resources.lib())?.build()?;
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

#[derive(Error, Debug)]
enum GuiError {
    #[error("Resource load error : {0}")]
    Resources(ResourcesError),
    #[error("Error during map load : {0}")]
    MapReader(MapReaderError),
    #[error("Running error : {0}")]
    RunGame(GameError),
    #[error("Error during input send : {0}")]
    SendInput(SendError<Vec<InputMessage>>),
    #[error("Network error : {0}")]
    Network(NetworkError),
    #[error("Embedded server error : {0}")]
    EmbeddedServer(EmbeddedServerError),
}

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

impl From<ResourcesError> for GuiError {
    fn from(error: ResourcesError) -> Self {
        Self::Resources(error)
    }
}
