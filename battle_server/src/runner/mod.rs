use battle_core::{
    config::ServerConfig,
    message::{InputMessage, OutputMessage},
    state::battle::BattleState,
};
use crossbeam_channel::{Receiver, SendError, Sender};
use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

mod behavior;
mod fight;
mod gesture;
mod input;
mod message;
mod movement;
mod output;
mod phase;
mod physics;
mod react;
mod soldier;
mod tick;
mod update;
mod utils;
mod vehicle;
mod visibility;

const TARGET_CYCLE_DURATION_US: u64 = 16666;

pub struct Runner {
    config: ServerConfig,
    input: Receiver<Vec<InputMessage>>,
    output: Sender<Vec<OutputMessage>>,
    stop_required: Arc<AtomicBool>,
    last: Instant,
    battle_state: BattleState,
    frame_i: u64,
}

impl Runner {
    pub fn new(
        config: ServerConfig,
        input: Receiver<Vec<InputMessage>>,
        output: Sender<Vec<OutputMessage>>,
        stop_required: Arc<AtomicBool>,
        state: BattleState,
    ) -> Self {
        Self {
            config,
            input,
            output,
            stop_required,
            last: Instant::now(),
            battle_state: state,
            frame_i: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), RunnerError> {
        loop {
            if self.stop_required.load(Ordering::Relaxed) {
                println!("Stopping runner ...");
                break;
            }

            let frame_i = self.frame_i;
            puffin::profile_scope!("run", format!("frame {frame_i}"));
            puffin::GlobalProfiler::lock().new_frame();

            thread::sleep(self.sleep_duration());
            self.last = Instant::now();
            self.tick()?;
            self.frame_i += 1;
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
