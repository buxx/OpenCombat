use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
};

use oc_network::ToClient;
use oc_root::Client;
#[cfg(feature = "debug")]
use oc_root::side::Side;

#[cfg(feature = "tracker")]
use crate::tracker::Tracker;
use crate::{routing::Listening, runner::Signal, state::State};

#[derive(Clone)]
pub struct Context<E: Client> {
    pub cpus: usize,
    pub state: Arc<State<E>>,
    pub output: Sender<(E, ToClient)>,
    #[cfg(feature = "tracker")]
    pub tracker: Tracker,
    #[cfg(feature = "debug")]
    direct: Option<(E, Side)>,
    /// When set, runner threads must stop
    stopped: Arc<AtomicBool>,
    /// Signals to runner (like battle end)
    signal: Sender<Signal>,
}

impl<E: Client> Context<E> {
    pub fn new(
        state: Arc<State<E>>,
        output: Sender<(E, ToClient)>,
        #[cfg(feature = "tracker")] tracker: Tracker,
        #[cfg(feature = "debug")] direct: Option<(E, Side)>,
        stopped: Arc<AtomicBool>,
        signal: Sender<Signal>,
    ) -> Self {
        let cpus = num_cpus::get();

        Self {
            cpus,
            state,
            output,
            #[cfg(feature = "tracker")]
            tracker,
            #[cfg(feature = "debug")]
            direct,
            stopped,
            signal,
        }
    }

    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Relaxed)
    }

    pub fn signal(&self, signal: Signal) {
        let _ = self.signal.send(signal);
    }

    pub fn broadcast<T>(&self, filter: Listening, messages: Vec<T>)
    where
        T: Clone + Into<ToClient>,
    {
        #[cfg(feature = "debug")]
        {
            // In cases like e2e tests, we want ensure gui receive all event. So, send all events
            if let Some((direct, side)) = &self.direct {
                if filter.match_side(*side) {
                    for message in &messages {
                        let message = message.clone().into();
                        self.output.send((direct.clone(), message)).unwrap() // TODO
                    }
                }
                return;
            }
        }

        let listeners = self.state.listeners();
        for listener in listeners.find(filter) {
            for message in &messages {
                let pkg = (listener.clone(), message.clone().into());
                self.output.send(pkg).unwrap() // TODO
            }
        }
    }
}
