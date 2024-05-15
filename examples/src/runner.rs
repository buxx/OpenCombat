use battle_core::{
    config::{
        GuiConfig, ServerConfig, DEFAULT_SERVER_PUB_ADDRESS, DEFAULT_SERVER_REP_ADDRESS,
        TARGET_CYCLE_DURATION_US,
    },
    deployment::Deployment,
    game::{control::MapControl, Side},
    map::Map,
    message::InputMessage,
    state::battle::{message::BattleStateMessage, BattleState},
};
use battle_gui::{
    debug::DebugTerrain,
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
    deployment: Deployment,
    begin: bool,
    target_cycle_duration: u64,
    debug_physics: bool,
    debug_terrain: DebugTerrain,
    when_first_copy_apply: Vec<EngineMessage>,
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
        let mut server_config = ServerConfig::default();
        server_config.target_cycle_duration_us = self.target_cycle_duration;
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

        engine_apply.push(EngineMessage::GuiState(GuiStateMessage::SetDebugTerrain(
            self.debug_terrain.clone(),
        )));

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
            self.when_first_copy_apply,
        )?;

        Ok(())
    }

    pub fn new(map: Map) -> Self {
        Self {
            map,
            deployment: Deployment::empty(),
            begin: false,
            target_cycle_duration: TARGET_CYCLE_DURATION_US,
            debug_physics: false,
            debug_terrain: DebugTerrain::None,
            when_first_copy_apply: vec![],
        }
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

    pub fn debug_terrain(mut self, value: DebugTerrain) -> Self {
        self.debug_terrain = value;
        self
    }

    pub fn target_cycle_duration(mut self, value: u64) -> Self {
        self.target_cycle_duration = value;
        self
    }

    pub fn when_first_copy_apply(mut self, value: Vec<EngineMessage>) -> Self {
        self.when_first_copy_apply = value;
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
