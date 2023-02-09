use std::fmt::Display;
use std::path::PathBuf;
use std::thread;

use battle_core::channel::Channel;
use battle_core::config::{ServerConfig, DEFAULT_SERVER_PUB_ADDRESS, DEFAULT_SERVER_REP_ADDRESS};
use battle_core::network::server::Server;
use battle_core::state::battle::builder::{BattleStateBuilder, BattleStateBuilderError};
use battle_server::runner::Runner;

#[derive(Debug)]
pub enum EmbeddedServerError {
    MissingMapName,
    MissingSituationName,
    StateBuilderError(BattleStateBuilderError),
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
            EmbeddedServerError::MissingSituationName => f.write_str("Missing situation name"),
            EmbeddedServerError::StateBuilderError(error) => {
                f.write_str(&format!("State builder error : {}", error))
            }
        }
    }
}

pub struct EmbeddedServer {
    resources: PathBuf,
    map_name: Option<String>,
    situation_name: Option<String>,
    server_rep_address: String,
    server_pub_address: String,
    server: bool,
}

impl EmbeddedServer {
    pub fn new(resources: &PathBuf) -> Self {
        Self {
            resources: resources.clone(),
            map_name: None,
            situation_name: None,
            server_rep_address: DEFAULT_SERVER_REP_ADDRESS.to_string(),
            server_pub_address: DEFAULT_SERVER_PUB_ADDRESS.to_string(),
            server: false,
        }
    }

    pub fn map_name(mut self, map_name: &str) -> Self {
        self.map_name = Some(map_name.to_string());
        self
    }

    pub fn situation_name(mut self, situation_name: &str) -> Self {
        self.situation_name = Some(situation_name.to_string());
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

    pub fn server(mut self, value: bool) -> Self {
        self.server = value;
        self
    }

    pub fn start(&self, channel: &Channel) -> Result<(), EmbeddedServerError> {
        let map_name = self
            .map_name
            .as_ref()
            .ok_or(EmbeddedServerError::MissingMapName)?;
        let situation_name = self
            .situation_name
            .as_ref()
            .ok_or(EmbeddedServerError::MissingSituationName)?;
        let config = ServerConfig::new();
        let state = BattleStateBuilder::new(&map_name, &self.resources)?
            .situation(&situation_name)
            .build();

        if self.server {
            let server_rep_address = self.server_rep_address.clone();
            let server_pub_address = self.server_pub_address.clone();
            let server_channel = channel.clone();
            thread::spawn(move || {
                println!("Start server");
                match Server::new(server_rep_address, server_pub_address, &server_channel).serve() {
                    Ok(_) => {
                        println!("Server finished to serve")
                    }
                    Err(error) => println!("ERROR : Server fail to serve : {}", error),
                }
            });
        }

        let runner_input = channel.input_receiver();
        let runner_output = channel.output_sender();
        thread::spawn(|| {
            println!("Start runner");
            match Runner::new(config, runner_input, runner_output, state).run() {
                Ok(_) => {
                    println!("Runner finished to run")
                }
                Err(error) => {
                    println!("ERROR : Runner fail to run : {}", error)
                }
            };
        });

        Ok(())
    }
}
