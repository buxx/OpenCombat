use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use battle_core::config::GuiConfig;
use battle_core::config::ServerConfig;
use battle_core::config::DEFAULT_SERVER_PUB_ADDRESS;
use battle_core::config::DEFAULT_SERVER_REP_ADDRESS;
use battle_core::deployment::Deployment;
use battle_core::game::control::MapControl;
use battle_core::game::Side;
use battle_core::map::Map;
use battle_core::message::InputMessage;
use battle_core::network::client::Client;
use battle_core::state::battle::message::BattleStateMessage;
use battle_core::state::battle::BattleState;

use crossbeam_channel::unbounded;
use ggez::event;
use oc_core::resources::Resources;
use oc_core::spawn::SpawnZoneName;
use structopt::StructOpt;

use crate::engine;
use crate::engine::message::EngineMessage;
use crate::graphics;
use crate::graphics::windowed_mode;
use crate::server::EmbeddedServer;
use crate::GuiError;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt()]
    pub map_name: String,

    #[structopt(parse(from_os_str))]
    pub deployment: PathBuf,

    #[structopt(long = "--embedded-server")]
    pub embedded_server: bool,

    #[structopt(long = "--server-rep-address", default_value = DEFAULT_SERVER_REP_ADDRESS)]
    pub server_rep_address: String,

    #[structopt(long = "--server-bind-address", default_value = DEFAULT_SERVER_PUB_ADDRESS)]
    pub server_pub_address: String,

    #[structopt(long = "side")]
    pub side: Side,

    #[structopt(long = "profile")]
    pub profile: bool,

    #[structopt(long = "--profile-address", default_value = "127.0.0.1:8585")]
    pub profile_address: String,

    #[structopt(long = "side-a-control")]
    pub a_control: Vec<SpawnZoneName>,

    #[structopt(long = "side-b-control")]
    pub b_control: Vec<SpawnZoneName>,

    #[structopt(long = "--init-sync")]
    pub init_sync: bool,
}

pub struct RunSettings {
    server_rep_address: String,
    server_pub_address: String,
    init_sync: bool,
    side: Side,
    embedded_server: bool,
}

impl RunSettings {
    pub fn new(
        server_rep_address: String,
        server_pub_address: String,
        init_sync: bool,
        side: Side,
        embedded_server: bool,
    ) -> Self {
        Self {
            server_rep_address,
            server_pub_address,
            init_sync,
            side,
            embedded_server,
        }
    }
}

impl From<Opt> for RunSettings {
    fn from(value: Opt) -> Self {
        Self {
            server_rep_address: value.server_rep_address.clone(),
            server_pub_address: value.server_pub_address.clone(),
            init_sync: value.init_sync,
            side: value.side,
            embedded_server: value.embedded_server,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    settings: RunSettings,
    config: GuiConfig,
    server_config: ServerConfig,
    a_control: MapControl,
    b_control: MapControl,
    map: Map,
    resources: Resources,
    deployment: Deployment,
    battle_state: BattleState,
    force_server_map: bool,
    inputs: Vec<InputMessage>,
    engine_apply: Vec<EngineMessage>,
) -> Result<(), GuiError> {
    let sync_required = Arc::new(AtomicBool::new(true));
    let stop_required: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let ready_message = if settings.side == Side::A {
        InputMessage::BattleState(BattleStateMessage::SetAConnected(true))
    } else {
        InputMessage::BattleState(BattleStateMessage::SetBConnected(true))
    };

    let (input_sender, output_receiver) = if settings.embedded_server {
        let (input_sender, input_receiver) = unbounded();
        let (output_sender, output_receiver) = unbounded();

        if let Err(error) = EmbeddedServer::new(
            &resources.lib(),
            input_receiver,
            output_sender,
            stop_required.clone(),
        )
        // FIXME map instead ? check
        .map_name(map.name())
        .force_map({
            if force_server_map {
                Some(map.clone())
            } else {
                None
            }
        })
        .server_rep_address(&settings.server_rep_address)
        .server_pub_address(&settings.server_pub_address)
        .start()
        {
            return Err(GuiError::EmbeddedServer(error));
        }

        (input_sender, output_receiver)
    } else {
        let (input_sender, input_receiver) = unbounded();
        let (output_sender, output_receiver) = unbounded();

        Client::new(
            settings.server_rep_address.clone(),
            settings.server_pub_address.clone(),
            input_sender.clone(),
            input_receiver,
            output_sender,
            output_receiver.clone(),
            sync_required.clone(),
        )
        .connect()?;

        (input_sender, output_receiver)
    };

    // These messages will initialize the battle state
    // Then, the RequireCompleteSync permit client to be same state than server
    if settings.init_sync {
        input_sender.send(
            [
                vec![
                    InputMessage::LoadDeployment(deployment),
                    InputMessage::LoadControl((a_control.clone(), b_control.clone())),
                    InputMessage::RequireCompleteSync,
                    ready_message,
                ],
                inputs,
            ]
            .concat(),
        )?;
    } else {
        input_sender.send(
            [
                vec![InputMessage::RequireCompleteSync, ready_message],
                inputs,
            ]
            .concat(),
        )?;
    }

    let mut context_builder =
        ggez::ContextBuilder::new("Open Combat", "Bastien Sevajol").window_mode(windowed_mode());
    for resource_path in resources.resources_paths_abs() {
        context_builder = context_builder.add_resource_path(resource_path);
    }
    let (mut context, event_loop) = context_builder.build()?;
    let graphics =
        graphics::Graphics::new(&mut context, &map, &server_config, &a_control, &b_control)?;
    let engine = engine::Engine::new(
        &mut context,
        &settings.side,
        config,
        server_config,
        input_sender,
        output_receiver,
        graphics,
        battle_state,
        sync_required,
        stop_required.clone(),
        a_control,
        b_control,
        engine_apply,
    )?;

    // FIXME BS NOW : Closing GUI don't close thread correctly and keep process running
    println!("Start Gui");
    event::run(context, event_loop, engine)
}
