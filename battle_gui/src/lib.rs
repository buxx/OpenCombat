pub mod audio;
pub mod debug;
pub mod engine;
pub mod error;
pub mod graphics;
pub mod physics;
pub mod run;
pub mod saves;
pub mod server;
pub mod ui;
pub mod utils;

use battle_core::deployment::DeploymentReaderError;
use battle_core::map::reader::MapReaderError;
use battle_core::message::InputMessage;
use battle_core::network::error::NetworkError;
use battle_core::state::battle::builder::BattleStateBuilderError;

use crossbeam_channel::SendError;
use ggez::GameError;
use oc_core::resources::ResourcesError;
use server::EmbeddedServerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GuiError {
    #[error("Resource load error : {0}")]
    Resources(ResourcesError),
    #[error("Deployment load error : {0}")]
    Deployment(DeploymentReaderError),
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
    #[error("Battle state builder error : {0}")]
    BattleStateBuilderError(BattleStateBuilderError),
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

impl From<DeploymentReaderError> for GuiError {
    fn from(error: DeploymentReaderError) -> Self {
        Self::Deployment(error)
    }
}

impl From<BattleStateBuilderError> for GuiError {
    fn from(error: BattleStateBuilderError) -> Self {
        Self::BattleStateBuilderError(error)
    }
}
