use std::{env::current_exe, path::PathBuf, process::Command};

use anyhow::{Context, Error, Result};

pub struct BattleLauncher {
    executable_path: PathBuf,
    map_name: String,
    deployment: PathBuf,
    embedded_server: bool,
    server_rep_address: String,
    server_bind_address: String,
    side: String,
    side_a_controls: Vec<String>,
    side_b_controls: Vec<String>,
}

impl BattleLauncher {
    pub fn new(map_name: &str, deployment: &PathBuf, side: &str) -> Result<Self> {
        let executable_path = current_exe().context("Retrieve current executable path")?;
        let executable_path = executable_path
            .parent()
            .context("Retrieve current executable path parent")?;
        Ok(Self {
            executable_path: executable_path.to_path_buf(),
            map_name: map_name.to_string(),
            deployment: deployment.clone(),
            embedded_server: true,
            server_rep_address: "tcp://0.0.0.0:4255".to_string(),
            server_bind_address: "tcp://0.0.0.0:4256".to_string(),
            side: side.to_string(),
            side_a_controls: vec![],
            side_b_controls: vec![],
        })
    }

    pub fn executable_path(mut self, value: PathBuf) -> Self {
        self.executable_path = value;
        self
    }

    pub fn map_name(mut self, value: String) -> Self {
        self.map_name = value;
        self
    }

    pub fn deployment(mut self, value: PathBuf) -> Self {
        self.deployment = value;
        self
    }

    pub fn embedded_server(mut self, value: bool) -> Self {
        self.embedded_server = value;
        self
    }

    pub fn server_rep_address(mut self, value: String) -> Self {
        self.server_rep_address = value;
        self
    }

    pub fn server_bind_address(mut self, value: String) -> Self {
        self.server_bind_address = value;
        self
    }

    pub fn side(mut self, value: String) -> Self {
        self.side = value;
        self
    }

    pub fn side_a_controls(mut self, value: Vec<String>) -> Self {
        self.side_a_controls = value;
        self
    }

    pub fn side_b_controls(mut self, value: Vec<String>) -> Self {
        self.side_b_controls = value;
        self
    }

    pub fn launch(&self) -> Result<()> {
        let embedded_server = if self.embedded_server {
            vec!["--embedded-server"]
        } else {
            vec![]
        };
        let server_rep_address = &format!("--server-rep-address={}", self.server_rep_address);
        let server_bind_address = &format!("--server-bind-address={}", self.server_bind_address);
        let side = &format!("--side={}", self.side);
        let side_a_control = self
            .side_a_controls
            .iter()
            .map(|c| format!("--side-a-control={}", c))
            .collect::<Vec<String>>();
        let side_b_control = self
            .side_b_controls
            .iter()
            .map(|c| format!("--side-b-control={}", c))
            .collect::<Vec<String>>();

        let mut command =
            Command::new(self.executable_path.join(self.battle_gui_executable_name()));
        let command = command
            .arg(&self.map_name)
            .arg(&self.deployment)
            .args(embedded_server)
            .arg(server_rep_address)
            .arg(server_bind_address)
            .arg(side)
            .args(side_a_control)
            .args(side_b_control);

        let command_line = format!("{:?}", command);
        command
            .spawn()
            .context(format!("Execute command line '{}'", command_line))?;

        Ok(())
    }

    fn battle_gui_executable_name(&self) -> &str {
        if cfg!(target_os = "windows") {
            "battle_gui.exe"
        } else {
            "battle_gui"
        }
    }
}
