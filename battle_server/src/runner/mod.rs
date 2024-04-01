pub mod worker;
use battle_core::{
    config::ServerConfig,
    message::{InputMessage, OutputMessage},
    state::battle::{
        message::{BattleStateMessage, SoldierMessage},
        BattleState,
    },
    types::SoldierIndex,
};
use crossbeam_channel::{Receiver, SendError, Sender};
use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock, RwLockReadGuard,
    },
    thread,
    time::{Duration, Instant},
};

use crate::runner::message::RunnerMessage;

use self::worker::Workers;

mod flag;
mod input;
mod message;
mod morale;
mod output;
mod phase;
mod physics;
mod react;
mod soldier;
mod utils;
mod victory;
mod visibility;

const TARGET_CYCLE_DURATION_US: u64 = 16666;

pub struct Runner {
    config: ServerConfig,
    workers: Workers,
    input: Receiver<Vec<InputMessage>>,
    output: Sender<Vec<OutputMessage>>,
    stop_required: Arc<AtomicBool>,
    last: Instant,
    battle_state: Arc<RwLock<BattleState>>,
}

impl Runner {
    pub fn new(
        config: ServerConfig,
        workers: Workers,
        input: Receiver<Vec<InputMessage>>,
        output: Sender<Vec<OutputMessage>>,
        stop_required: Arc<AtomicBool>,
        state: Arc<RwLock<BattleState>>,
    ) -> Self {
        Self {
            config,
            workers,
            input,
            output,
            stop_required,
            last: Instant::now(),
            battle_state: state,
        }
    }

    pub fn run(&mut self) -> Result<(), RunnerError> {
        loop {
            if self.stop_required.load(Ordering::Relaxed) {
                println!("Stopping runner ...");
                break;
            }
            let frame_i = {
                let battle_state = self.battle_state();
                *battle_state.frame_i()
            };

            puffin::profile_scope!("run", format!("frame {frame_i}"));
            puffin::GlobalProfiler::lock().new_frame();

            self.workers.trigger(self, &frame_i);
            thread::sleep(self.sleep_duration());
            self.last = Instant::now();

            let mut messages = vec![RunnerMessage::BattleState(
                BattleStateMessage::IncrementFrameI,
            )];
            messages.extend(self.tick_phase());
            messages.extend(self.tick_morale());
            messages.extend(self.tick_victory());
            messages.extend(self.tick_flags());
            messages.extend(self.tick_update_squad_leaders());
            messages.extend(self.tick_feeling_decreasing_soldiers());
            messages.extend(self.tick_physics());
            messages.extend(self.workers.messages()?);

            self.react(&messages);
            self.clean();
            self.outputs(&messages)?;
        }

        Ok(())
    }

    fn sleep_duration(&self) -> Duration {
        let elapsed = self.last.elapsed().as_micros() as u64;
        if elapsed > TARGET_CYCLE_DURATION_US {
            Duration::from_micros(0)
        } else {
            Duration::from_micros(TARGET_CYCLE_DURATION_US - elapsed)
        }
    }

    pub fn clean(&mut self) {
        // self.battle_state.clean(None);
    }

    pub fn battle_state(&self) -> RwLockReadGuard<'_, BattleState> {
        // FIXME unwrap
        self.battle_state.read().unwrap()
    }

    // TODO: move this
    pub fn tick_update_squad_leaders(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_update_squad_leaders");
        let mut messages = vec![];
        let battle_state = self.battle_state();

        let tick_update = battle_state.frame_i() % self.config.squad_leaders_update_freq() == 0;
        if tick_update {
            for squad_uuid in battle_state.squads().keys() {
                let squad = battle_state.squad(*squad_uuid);
                let leader = battle_state.soldier(squad.leader());

                if !leader.can_be_leader() {
                    if let Some(member) = squad
                        .subordinates()
                        .iter()
                        .map(|s| battle_state.soldier(**s))
                        .find(|s| s.can_be_leader())
                    {
                        messages.push(RunnerMessage::BattleState(
                            BattleStateMessage::SetSquadLeader(*squad_uuid, member.uuid()),
                        ))
                    }
                }
            }
        }
        messages
    }

    // TODO: move this
    pub fn tick_feeling_decreasing_soldiers(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_feeling_decreasing_soldiers");
        let mut messages = vec![];
        let tick_feeling_decreasing =
            self.battle_state().frame_i() % self.config.feeling_decreasing_freq() == 0
                && self.battle_state().phase().is_battle();

        if tick_feeling_decreasing {
            messages.extend((0..self.battle_state().soldiers().len()).map(|i| {
                RunnerMessage::BattleState(BattleStateMessage::Soldier(
                    SoldierIndex(i),
                    SoldierMessage::DecreaseUnderFire,
                ))
            }));
        }

        messages
    }
}

#[derive(Debug)]
pub enum RunnerError {
    InputChannelClosed,
    Output(SendError<Vec<OutputMessage>>),
}

impl From<SendError<Vec<OutputMessage>>> for RunnerError {
    fn from(error: SendError<Vec<OutputMessage>>) -> Self {
        Self::Output(error)
    }
}

impl Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunnerError::InputChannelClosed => f.write_str("Input channel closed"),
            RunnerError::Output(error) => f.write_str(&format!("Output error : {}", error)),
        }
    }
}
