use std::{
    sync::{Arc, RwLock},
    thread,
};

use battle_core::{config::ServerConfig, state::battle::BattleState};
use crossbeam_channel::{unbounded, Receiver, Sender};

use super::{
    message::RunnerMessage, soldier::SoldierRunner, visibility::VisibilityRunner, Runner,
    RunnerError,
};

pub struct Tick;

pub struct Workers {
    config: ServerConfig,
    battle_state: Arc<RwLock<BattleState>>,
    input_animate_soldiers_reader: Receiver<Tick>,
    input_animate_soldiers_sender: Sender<Tick>,
    input_update_soldiers_reader: Receiver<Tick>,
    input_update_soldiers_sender: Sender<Tick>,
    input_visibilities_reader: Receiver<Tick>,
    input_visibilities_sender: Sender<Tick>,
    output_reader: Receiver<Vec<RunnerMessage>>,
    output_sender: Sender<Vec<RunnerMessage>>,
}

impl Workers {
    pub fn new(config: ServerConfig, battle_state: Arc<RwLock<BattleState>>) -> Self {
        let (input_animate_soldiers_sender, input_animate_soldiers_reader) = unbounded();
        let (input_update_soldiers_sender, input_update_soldiers_reader) = unbounded();
        let (input_visibilities_sender, input_visibilities_reader) = unbounded();
        let (output_sender, output_reader) = unbounded();
        Self {
            config,
            battle_state,
            input_animate_soldiers_reader,
            input_animate_soldiers_sender,
            input_update_soldiers_reader,
            input_update_soldiers_sender,
            input_visibilities_reader,
            input_visibilities_sender,
            output_reader,
            output_sender,
        }
    }

    pub fn start(&self) {
        let config = self.config.clone();
        let battle_state = self.battle_state.clone();
        let output_sender = self.output_sender.clone();
        let input_animate_soldiers_reader = self.input_animate_soldiers_reader.clone();
        thread::spawn(move || {
            while let Ok(_) = input_animate_soldiers_reader.recv() {
                let messages = SoldierRunner::new(config.clone(), battle_state.clone())
                    .tick_animate_soldiers();
                output_sender.send(messages);
            }
        });

        let config = self.config.clone();
        let battle_state = self.battle_state.clone();
        let output_sender = self.output_sender.clone();
        let input_update_soldiers_reader = self.input_update_soldiers_reader.clone();
        thread::spawn(move || {
            while let Ok(_) = input_update_soldiers_reader.recv() {
                let messages =
                    SoldierRunner::new(config.clone(), battle_state.clone()).tick_update_soldiers();
                output_sender.send(messages);
            }
        });

        let config = self.config.clone();
        let battle_state = self.battle_state.clone();
        let output_sender = self.output_sender.clone();
        let input_visibilities_reader = self.input_visibilities_reader.clone();
        thread::spawn(move || {
            while let Ok(_) = input_visibilities_reader.recv() {
                let messages =
                    VisibilityRunner::new(config.clone(), battle_state.clone()).tick_visibilities();
                output_sender.send(messages);
            }
        });
    }

    pub fn trigger(&self, runner: &Runner, frame_i: &u64) -> Result<(), RunnerError> {
        let tick_animate = frame_i % self.config.soldier_animate_freq() == 0
            && runner.battle_state().phase().is_battle();
        let tick_update = frame_i % self.config.soldier_update_freq() == 0;
        let tick_visibility = frame_i % self.config.visibility_update_freq() == 0
            && runner.battle_state().phase().is_battle();

        if tick_animate {
            self.input_animate_soldiers_sender.send(Tick);
        }
        if tick_update {
            self.input_update_soldiers_sender.send(Tick);
        }
        if tick_visibility {
            self.input_visibilities_sender.send(Tick);
        }

        Ok(())
    }

    pub fn messages(&self) -> Result<Vec<RunnerMessage>, RunnerError> {
        let mut messages = vec![];

        while let Ok(messages_) = self.output_reader.try_recv() {
            messages.extend::<Vec<RunnerMessage>>(messages_)
        }

        Ok(messages)
    }
}
