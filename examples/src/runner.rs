use battle_core::{
    config::{GuiConfig, ServerConfig, DEFAULT_SERVER_PUB_ADDRESS, DEFAULT_SERVER_REP_ADDRESS},
    deployment::Deployment,
    game::{control::MapControl, Side},
    map::Map,
    message::InputMessage,
    state::battle::{message::BattleStateMessage, BattleState},
};
use battle_gui::{
    engine::message::{EngineMessage, GuiStateMessage},
    run::{run, RunSettings},
    GuiError,
};

use oc_core::{
    resources::{Resources, ResourcesError},
    spawn::SpawnZoneName,
};
use thiserror::Error;

pub struct Runner {
    map: Map,
    expire: Option<u64>, // FIXME BS NOW: use it
    deployment: Deployment,
    begin: bool,
    debug_physics: bool,
}

impl Runner {
    pub fn run(self) -> Result<(), RunnerError> {
        let settings = RunSettings::new(
            DEFAULT_SERVER_REP_ADDRESS.into(),
            DEFAULT_SERVER_PUB_ADDRESS.into(),
            true,
            Side::A,
            true,
        );
        let config = GuiConfig {
            target_fps: 60,
            interiors_update_freq: 60,
        };
        let server_config = ServerConfig::default();
        let (a_control, b_control) = (
            MapControl::new(vec![SpawnZoneName::All]),
            MapControl::new(vec![SpawnZoneName::All]),
        );
        let resources = Resources::new()?.ensure()?;
        let battle_state = BattleState::empty(&self.map);

        let mut inputs = vec![];
        let mut engine_apply = vec![];

        if self.begin {
            inputs.extend(vec![
                InputMessage::BattleState(BattleStateMessage::SetAReady(true)),
                InputMessage::BattleState(BattleStateMessage::SetBReady(true)),
            ]);
            engine_apply.extend(vec![EngineMessage::GuiState(GuiStateMessage::SetIntroAck(
                true,
            ))]);
        }

        if self.debug_physics {
            engine_apply.extend(vec![EngineMessage::GuiState(
                GuiStateMessage::SetDebugPhysicsArea(true),
            )])
        }

        run(
            settings,
            config,
            server_config,
            a_control,
            b_control,
            self.map,
            resources,
            self.deployment.clone(),
            battle_state,
            true,
            inputs,
            engine_apply,
        )?;

        Ok(())
    }

    pub fn new(map: Map) -> Self {
        Self {
            map,
            expire: None,
            deployment: Deployment::empty(),
            begin: false,
            debug_physics: false,
        }
    }

    pub fn expire(mut self, value: Option<u64>) -> Self {
        self.expire = value;
        self
    }

    pub fn deployment(mut self, value: Deployment) -> Self {
        self.deployment = value;
        self
    }

    pub fn begin(mut self, value: bool) -> Self {
        self.begin = value;
        self
    }

    pub fn debug_physics(mut self, value: bool) -> Self {
        self.debug_physics = value;
        self
    }
}

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("Gui error: {0}")]
    GuiError(#[from] GuiError),
    #[error("Resources error: {0}")]
    ResourcesError(#[from] ResourcesError),
}
