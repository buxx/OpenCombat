use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

use battle_core::config::{ServerConfig, DEFAULT_SERVER_PUB_ADDRESS, DEFAULT_SERVER_REP_ADDRESS};
use battle_core::message::{InputMessage, OutputMessage};
use battle_core::network::error::NetworkError;
use battle_core::network::server::Server;
use battle_core::state::battle::builder::{BattleStateBuilder, BattleStateBuilderError};
use battle_server::runner::Runner;
use crossbeam_channel::{unbounded, Receiver, Sender};

type ServerChannel = (Sender<Vec<OutputMessage>>, Receiver<Vec<InputMessage>>);
type RunnerChannel = (Sender<Vec<InputMessage>>, Receiver<Vec<OutputMessage>>);

#[derive(Debug)]
pub enum EmbeddedServerError {
    MissingMapName,
    StateBuilderError(BattleStateBuilderError),
    Network(NetworkError),
}

impl From<BattleStateBuilderError> for EmbeddedServerError {
    fn from(error: BattleStateBuilderError) -> Self {
        Self::StateBuilderError(error)
    }
}

impl Display for EmbeddedServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddedServerError::MissingMapName => f.write_str("Missing map name"),
            EmbeddedServerError::StateBuilderError(error) => {
                f.write_str(&format!("State builder error : {}", error))
            }
            EmbeddedServerError::Network(error) => {
                f.write_str(&format!("Network serve error : {}", error))
            }
        }
    }
}

pub struct EmbeddedServer {
    resources: PathBuf,
    map_name: Option<String>,
    server_rep_address: String,
    server_pub_address: String,
    gui_input_receiver: Receiver<Vec<InputMessage>>,
    gui_output_sender: Sender<Vec<OutputMessage>>,
    stop_required: Arc<AtomicBool>,
}

impl EmbeddedServer {
    pub fn new(
        resources: &Path,
        gui_input_receiver: Receiver<Vec<InputMessage>>,
        gui_output_sender: Sender<Vec<OutputMessage>>,
        stop_required: Arc<AtomicBool>,
    ) -> Self {
        Self {
            resources: resources.to_path_buf(),
            map_name: None,
            server_rep_address: DEFAULT_SERVER_REP_ADDRESS.to_string(),
            server_pub_address: DEFAULT_SERVER_PUB_ADDRESS.to_string(),
            gui_input_receiver,
            gui_output_sender,
            stop_required,
        }
    }

    pub fn map_name(mut self, map_name: &str) -> Self {
        self.map_name = Some(map_name.to_string());
        self
    }

    pub fn server_rep_address(mut self, address: &str) -> Self {
        self.server_rep_address = address.to_string();
        self
    }

    pub fn server_pub_address(mut self, address: &str) -> Self {
        self.server_pub_address = address.to_string();
        self
    }

    fn start_runner(&self) -> Result<RunnerChannel, EmbeddedServerError> {
        let (runner_input_sender, runner_input_receiver) = unbounded();
        let (runner_output_sender, runner_output_receiver) = unbounded();

        let map_name = self
            .map_name
            .as_ref()
            .ok_or(EmbeddedServerError::MissingMapName)?;
        let config = ServerConfig::default();
        let state = BattleStateBuilder::new(map_name, self.resources.clone()).build()?;

        let stop_required_ = self.stop_required.clone();
        thread::Builder::new()
            .name("runner".to_string())
            .spawn(|| {
                println!("Start runner");
                match Runner::new(
                    config,
                    runner_input_receiver,
                    runner_output_sender,
                    stop_required_,
                    state,
                )
                .run()
                {
                    Ok(_) => {
                        println!("Runner finished to run")
                    }
                    Err(error) => {
                        println!("ERROR : Runner fail to run : {}", error)
                    }
                };
            })
            .unwrap();

        Ok((runner_input_sender, runner_output_receiver))
    }

    fn start_server(&self) -> Result<ServerChannel, EmbeddedServerError> {
        let server_rep_address = self.server_rep_address.clone();
        let server_pub_address = self.server_pub_address.clone();
        let (server_input_sender, server_input_receiver) = unbounded();
        let (server_output_sender, server_output_receiver) = unbounded();

        println!("Start server");
        if let Err(error) = Server::new(
            server_rep_address,
            server_pub_address,
            server_output_receiver,
            server_input_sender,
            self.stop_required.clone(),
        )
        .serve()
        {
            return Err(EmbeddedServerError::Network(error));
        };

        Ok((server_output_sender, server_input_receiver))
    }

    pub fn start(&self) -> Result<(), EmbeddedServerError> {
        let (runner_input_sender, runner_output_receiver) = self.start_runner()?;
        let (server_output_sender, server_input_receiver) = self.start_server()?;

        let gui_input_receiver_ = self.gui_input_receiver.clone();
        let runner_input_sender_ = runner_input_sender.clone();
        thread::Builder::new()
            .name("emb_gui_inputs_bridge".to_string())
            .spawn(move || {
                while let Ok(messages) = gui_input_receiver_.recv() {
                    if let Err(error) = runner_input_sender_.send(messages) {
                        println!(
                            "Error during transmit gui input messages to runner : {}",
                            error
                        )
                    }
                }

                println!("Gui input bridge finished");
            })
            .expect("Thread must be builded correctly");

        let gui_output_sender_ = self.gui_output_sender.clone();
        thread::Builder::new()
            .name("emb_runner_outputs_bridge".to_string())
            .spawn(move || {
                while let Ok(messages) = runner_output_receiver.recv() {
                    if let Err(error) = gui_output_sender_.send(messages.clone()) {
                        println!(
                            "Error during transmit runner output messages to gui : {}",
                            error
                        )
                    };
                    if let Err(error) = server_output_sender.send(messages) {
                        println!(
                            "Error during transmit runner output messages to server : {}",
                            error
                        )
                    };
                }

                println!("Runner output bridge finished");
            })
            .expect("Thread must be builded correctly");

        thread::Builder::new()
            .name("emb_server_inputs_bridge".to_string())
            .spawn(move || {
                while let Ok(messages) = server_input_receiver.recv() {
                    if let Err(error) = runner_input_sender.send(messages) {
                        println!(
                            "Error during transmit server input messages to runner : {}",
                            error
                        )
                    }
                }

                println!("Server input bridge finished");
            })
            .expect("Thread must be builded correctly");

        Ok(())
    }
}
