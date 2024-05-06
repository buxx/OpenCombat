use battle_core::{
    config::{GuiConfig, ServerConfig, DEFAULT_SERVER_PUB_ADDRESS, DEFAULT_SERVER_REP_ADDRESS},
    deployment::Deployment,
    game::{control::MapControl, Side},
    map::Map,
    state::battle::BattleState,
};
use battle_gui::{
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
        )?;

        Ok(())
    }

    pub fn new(map: Map) -> Self {
        Self {
            map,
            expire: None,
            deployment: Deployment::empty(),
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
}

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("Gui error: {0}")]
    GuiError(#[from] GuiError),
    #[error("Resources error: {0}")]
    ResourcesError(#[from] ResourcesError),
}
