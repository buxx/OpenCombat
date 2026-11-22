use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, RecvTimeoutError, Sender, channel},
    },
    time::{Duration, Instant},
};

use oc_network::{ToClient, ToServer};
use oc_replay::{Frame, Reader};

use crate::network::input::NetworkMessage;

pub enum Control {
    Seek(Duration),
    Speed(f32),
    Pause(bool),
}

#[derive(Debug, Clone, Default)]
pub struct Status {
    /// Record time of the last sent message
    pub position: Duration,
    /// Record time of the last message
    pub total: Duration,
    /// Messages are being sent at once (after a jump)
    pub seeking: bool,
    /// Replay is ended
    pub finished: bool,
    /// Play speed
    pub speed: f32,
    /// Used paused or not
    pub paused: bool,
}

/// Follow and control a running replay
#[derive(Clone)]
pub struct Handle {
    status: Arc<Mutex<Status>>,
    control: Sender<Control>,
}

impl Handle {
    pub fn status(&self) -> Status {
        self.status.lock().expect("Assume mutex").clone()
    }

    pub fn seek(&self, at: Duration) {
        self.status.lock().expect("Assume mutex").seeking = true;
        let _ = self.control.send(Control::Seek(at));
    }

    pub fn set_speed(&self, speed: f32) {
        let _ = self.control.send(Control::Speed(speed));
    }

    pub fn set_paused(&self, paused: bool) {
        let _ = self.control.send(Control::Pause(paused));
    }
}

struct Clock {
    position: Duration,
    since: Instant,
    speed: f32,
    paused: bool,
}

impl Clock {
    fn new(position: Duration, speed: f32) -> Self {
        Self {
            position,
            since: Instant::now(),
            speed,
            paused: false,
        }
    }

    fn now(&self) -> Duration {
        if self.paused {
            return self.position;
        }
        self.position + self.since.elapsed().mul_f32(self.speed)
    }

    fn until(&self, at: Duration) -> Option<Duration> {
        if self.paused {
            return None;
        }
        let at = self.since + at.saturating_sub(self.position).div_f32(self.speed);
        Some(at.saturating_duration_since(Instant::now()))
    }

    fn jump(&mut self, at: Duration) {
        self.position = at;
        self.since = Instant::now();
    }

    fn set_speed(&mut self, speed: f32) {
        self.jump(self.now());
        self.speed = speed;
    }

    fn set_paused(&mut self, paused: bool) {
        self.jump(self.now());
        self.paused = paused;
    }
}

pub fn start(
    path: PathBuf,
    speed: f32,
    input: Sender<NetworkMessage>,
    output: Receiver<ToServer>,
) -> Result<Handle, String> {
    let total =
        Reader::duration(&path).map_err(|e| format!("Open record {}: {e}", path.display()))?;
    let status = Arc::new(Mutex::new(Status {
        total,
        speed,
        ..Default::default()
    }));
    let (control_tx, control_rx) = channel();

    let mut player = Player {
        path,
        speed,
        input,
        output,
        control: control_rx,
        status: status.clone(),
    };
    std::thread::spawn(move || {
        if let Err(error) = player.play() {
            tracing::error!("Replay: {error}");
        }
    });

    Ok(Handle {
        status,
        control: control_tx,
    })
}

struct Player {
    path: PathBuf,
    speed: f32,
    input: Sender<NetworkMessage>,
    output: Receiver<ToServer>,
    control: Receiver<Control>,
    status: Arc<Mutex<Status>>,
}

impl Player {
    fn play(&mut self) -> Result<(), String> {
        let mut reader = open(&self.path)?;
        self.input
            .send(NetworkMessage::Connected)
            .map_err(|_| "Gui closed")?;

        let Some(Frame {
            message: config @ ToClient::GameConfig(_),
            ..
        }) = next(&mut reader)?
        else {
            return Err("Record must start by game config".to_string());
        };
        self.send(config)?;

        // Get and send all messages from WorldResume to WorldResume
        self.wait(|m| matches!(m, ToServer::RequestInit(_)))?;
        for frame in init_frames(&mut reader)? {
            self.send(frame.message)?;
        }

        // consider replay start when receive ListenRegion
        self.wait(|m| matches!(m, ToServer::ListenRegion(_)))?;
        tracing::info!("Start replay (speed x{})", self.speed);

        // First frame defines the replay start
        let mut pending = next(&mut reader)?;
        let start = pending.as_ref().map(|f| f.elapsed).unwrap_or_default();
        let mut clock = Clock::new(start, self.speed);

        loop {
            if pending.is_none() {
                pending = next(&mut reader)?;
            }
            if pending.is_none() {
                // Record is finished, but user can still jump
                self.update(|s| s.finished = true);
            }

            let until = pending.as_ref().and_then(|f| clock.until(f.elapsed));
            let control = match until {
                Some(delay) => match self.control.recv_timeout(delay) {
                    Ok(control) => Some(control),
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => {
                        std::thread::sleep(delay);
                        None
                    }
                },
                // Paused or finished: wait an order
                None => match self.control.recv() {
                    Ok(control) => Some(control),
                    Err(_) => return Ok(()),
                },
            };

            match control {
                Some(Control::Seek(at)) => {
                    pending = self.seek(&mut reader, at, pending.take())?;
                    clock.jump(at);
                }
                Some(Control::Speed(speed)) => {
                    tracing::info!("Replay speed x{speed}");
                    clock.set_speed(speed);
                    self.update(|s| s.speed = speed);
                }
                Some(Control::Pause(paused)) => {
                    tracing::info!("Replay paused: {paused}");
                    clock.set_paused(paused);
                    self.update(|s| s.paused = paused);
                }
                None => {
                    let Some(frame) = pending.take() else {
                        continue;
                    };
                    // Gui messages are ignored, but must be consumed
                    self.output.try_iter().for_each(drop);
                    let elapsed = frame.elapsed;
                    self.send(frame.message)?;
                    self.update(|s| s.position = elapsed);
                }
            }
        }
    }

    /// Send at once messages until record time `at`. `pending` is the next frame not sent yet.
    /// Return the next frame to send.
    fn seek(
        &self,
        reader: &mut Reader,
        at: Duration,
        pending: Option<Frame>,
    ) -> Result<Option<Frame>, String> {
        let position = self.status().position;
        tracing::info!("Replay jump to {at:?} (from {position:?})");
        self.update(|s| {
            s.seeking = true;
            s.finished = false;
        });

        let mut pending = pending;
        if at < position {
            // Gui must forget everything, then receive messages since start
            self.input
                .send(NetworkMessage::ReplayReset)
                .map_err(|_| "Gui closed")?;
            *reader = open(&self.path)?;
            next(reader)?; // game config
            init_frames(reader)?; // world resume is still valid
            pending = None;
        }

        let next_ = loop {
            let frame = match pending.take() {
                Some(frame) => Some(frame),
                None => next(reader)?,
            };
            let Some(frame) = frame else { break None };
            if frame.elapsed > at {
                break Some(frame);
            }
            if !transient(&frame.message) {
                self.send(frame.message)?;
            }
        };

        self.update(|s| {
            s.seeking = false;
            s.position = at;
        });

        Ok(next_)
    }

    fn send(&self, message: ToClient) -> Result<(), String> {
        self.input
            .send(NetworkMessage::Message(Box::new(message)))
            .map_err(|_| "Gui closed".to_string())
    }

    fn status(&self) -> Status {
        self.status.lock().expect("Assume mutex").clone()
    }

    fn update(&self, f: impl FnOnce(&mut Status)) {
        f(&mut self.status.lock().expect("Assume mutex"))
    }

    /// Ignore gui messages until one matching `expected`
    fn wait(&self, expected: impl Fn(&ToServer) -> bool) -> Result<(), String> {
        loop {
            let message = self.output.recv().map_err(|_| "Gui closed".to_string())?;
            if expected(&message) {
                return Ok(());
            }
        }
    }
}

fn open(path: &Path) -> Result<Reader, String> {
    Reader::open(path).map_err(|e| format!("Open record {}: {e}", path.display()))
}

fn next(reader: &mut Reader) -> Result<Option<Frame>, String> {
    reader.next_frame().map_err(|e| e.to_string())
}

/// Frames following game config, until world resume (included)
fn init_frames(reader: &mut Reader) -> Result<Vec<Frame>, String> {
    let mut frames = vec![];

    loop {
        let Some(frame) = next(reader)? else {
            return Err("No world resume in record".to_string());
        };

        let resume = matches!(frame.message, ToClient::WorldResume(_));
        // Prepare all message
        frames.push(frame);

        // And return them when WorldResume received (WorldResume included)
        if resume {
            return Ok(frames);
        }
    }
}

/// Messages without lasting effect (skipped when jumping)
fn transient(message: &ToClient) -> bool {
    matches!(
        message,
        ToClient::Fx(_)
            | ToClient::Collision(..)
            | ToClient::Projectile(_)
            | ToClient::SupportExecuted(_)
    )
}

#[cfg(test)]
mod tests {
    use oc_network::GameConfig;
    use oc_replay::{Header, Source, Writer};
    use oc_root::{WorldConfig, battle::BattlePhase, files::Roots, side::BySide, side::Side};
    use oc_world::{meta::Meta, resume::WorldResume};

    use super::*;

    const TIMEOUT: Duration = Duration::from_secs(5);

    /// Game config and world resume at 0, then `Elapsed(n)` at n seconds (1 to 10)
    fn record(path: &Path) {
        let mod_ = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods/std1");
        let mod_ = oc_mod::Mod::load(&mod_, None).unwrap();
        let config = GameConfig::new(
            WorldConfig::new(2, 2),
            mod_,
            Meta::default(),
            oc_root::static_::StaticSource::Local,
            true,
        );
        let resume = WorldResume {
            squads: vec![],
            spawns: BySide::default(),
            phase: BattlePhase::Deployment,
            support: vec![],
        };

        let source = Source::Local(Roots::under(Path::new("/tmp")).unwrap());
        let header = Header::from_config(Side::A, source, &config);
        let mut writer = Writer::create(path, &header).unwrap();
        writer
            .write(Duration::ZERO, &ToClient::GameConfig(Box::new(config)))
            .unwrap();
        writer
            .write(Duration::ZERO, &ToClient::WorldResume(resume))
            .unwrap();
        for n in 1..=10 {
            let at = Duration::from_secs(n);
            writer.write(at, &ToClient::Elapsed(at, at)).unwrap();
        }
        writer.flush().unwrap();
    }

    fn receive(input: &Receiver<NetworkMessage>) -> String {
        match input.recv_timeout(TIMEOUT).unwrap() {
            NetworkMessage::Connected => "connected".into(),
            NetworkMessage::ReplayReset => "reset".into(),
            NetworkMessage::Message(message) => match *message {
                ToClient::GameConfig(_) => "config".into(),
                ToClient::WorldResume(_) => "resume".into(),
                ToClient::Elapsed(at, _) => format!("{}", at.as_secs()),
                other => format!("{other:?}"),
            },
            other => format!("{other:?}"),
        }
    }

    #[test]
    fn seek_pause_and_speed() {
        let path = std::env::temp_dir().join(format!("oc_gui_replay_{}.ocrec", std::process::id()));
        record(&path);

        let (input_tx, input) = channel();
        let (output, output_rx) = channel();
        let handle = start(path.clone(), 1.0, input_tx, output_rx).unwrap();
        assert_eq!(handle.status().total, Duration::from_secs(10));

        let received = |n: usize| (0..n).map(|_| receive(&input)).collect::<Vec<_>>();
        assert_eq!(received(2), ["connected", "config"]);
        output.send(ToServer::Refresh).unwrap(); // ignored
        output
            .send(ToServer::RequestInit(oc_root::identity::Identity {
                uuid: uuid::Uuid::new_v4(),
                side: Side::A,
            }))
            .unwrap();
        assert_eq!(received(1), ["resume"]);
        output
            .send(ToServer::ListenRegion(oc_geo::region::WorldRegionIndex(0)))
            .unwrap();
        // First message defines replay start: sent immediately
        assert_eq!(received(1), ["1"]);

        // Forward: following messages until 5s at once, no reset
        handle.seek(Duration::from_secs(5));
        assert_eq!(received(4), ["2", "3", "4", "5"]);

        // Backward: reset, then messages since start until 2s at once
        handle.seek(Duration::from_secs(2));
        assert_eq!(received(3), ["reset", "1", "2"]);

        // Then, back to normal pace
        assert_eq!(received(1), ["3"]);

        // Paused: nothing is sent
        handle.set_paused(true);
        assert!(input.recv_timeout(Duration::from_millis(1500)).is_err());
        assert!(handle.status().paused);

        // Unpaused at x10: the 7 remaining seconds take 0.7s
        handle.set_speed(10.);
        handle.set_paused(false);
        let start = Instant::now();
        assert_eq!(received(7), ["4", "5", "6", "7", "8", "9", "10"]);
        assert!(start.elapsed() < Duration::from_millis(1500));
        assert_eq!(handle.status().speed, 10.);
        std::fs::remove_file(&path).ok();
    }
}
