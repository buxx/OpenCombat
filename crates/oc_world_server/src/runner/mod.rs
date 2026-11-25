use oc_root::format::RkyvFile;
use oc_utils::true_;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use oc_individual::{IndividualIndex, squad::SquadIndex};
use oc_network::{GameConfig, ToClient, ToServer};
use oc_root::side::Side;
use oc_root::{Client, battle::BattlePhase};
use oc_world::{resume::EndResume, snapshot};

#[cfg(feature = "tracker")]
use crate::tracker::Tracker;
use crate::{
    bridge::Event, config::ServerConfig, end::EndPolicy, individual, physics, routing::Listening,
    runner::input::Dealer, squad, state::State, utils::context::Context,
};

const WORLD_INTERVAL: Duration = Duration::from_secs(5);
/// AI decisions frequency
const AI_INTERVAL: Duration = Duration::from_millis(2755);
/// Smoke clouds fading precision
const SMOKE_INTERVAL: Duration = Duration::from_millis(250);
const SMOKE_MIN_INTERVAL: Duration = Duration::from_millis(20); // For e2e tests purposes
/// Watchers computed by visibilities thread between two releases of world locks
const VISIBILITIES_CHUNK: usize = 2;
/// Individuals/squads step
const STEP_BATCH: usize = 16;

pub mod explosion;
pub mod input;
pub mod support;
pub mod update;

/// Signals received by runner main loop
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    /// Stop requested from outside (like SIGINT)
    Stop,
    /// Battle ended
    End,
}

pub struct Runner<E: Client> {
    config: ServerConfig,
    state: Arc<State<E>>,
    output: Sender<(E, ToClient)>,
    #[cfg(feature = "tracker")]
    tracker: Tracker,
    #[cfg(feature = "debug")]
    /// When set, avoid listening filter when broadcast (to ensure vent receive in e2e tests)
    direct: Option<(E, Side)>,
    end: EndPolicy,
    /// Shared with all runner threads: when set, they stop
    stopped: Arc<AtomicBool>,
    /// Ais (generaly one, two for AIs fight in experiment pruposes)
    ais: Vec<crate::ai::AiConfig>,
    /// Runner react (SIGHINT or end of game)
    signal: (Sender<Signal>, Receiver<Signal>),
}

impl<E: Client> Runner<E> {
    pub fn embedded(
        config: ServerConfig,
        state: Arc<State<E>>,
        output: Sender<(E, ToClient)>,
        #[cfg_attr(not(feature = "debug"), allow(unused_variables))] gui: E,
        #[cfg_attr(not(feature = "debug"), allow(unused_variables))] side: Side,
        end: EndPolicy,
    ) -> Self {
        Self::new(
            config,
            state,
            output,
            #[cfg(feature = "tracker")]
            Tracker::default(),
            #[cfg(feature = "debug")]
            Some((gui, side)),
            end,
        )
    }

    pub fn new(
        config: ServerConfig,
        state: Arc<State<E>>,
        output: Sender<(E, ToClient)>,
        #[cfg(feature = "tracker")] tracker: Tracker,
        #[cfg(feature = "debug")] direct: Option<(E, Side)>,
        end: EndPolicy,
    ) -> Self {
        Self {
            config,
            state,
            output,
            #[cfg(feature = "tracker")]
            tracker,
            #[cfg(feature = "debug")]
            direct,
            end,
            stopped: Arc::new(AtomicBool::new(false)),
            ais: vec![],
            signal: channel(),
        }
    }

    pub fn with_ais(mut self, ais: impl IntoIterator<Item = crate::ai::AiConfig>) -> Self {
        self.ais.extend(ais);
        self
    }

    pub fn run(
        &self,
        input: Receiver<Event<E>>,
        ready: Sender<Result<(), String>>,
        stop: Receiver<()>,
    ) {
        self.listen_input(input);
        self.listen_stop(stop);

        let mut threads = vec![];
        threads.extend(self.start_physics());
        threads.extend(self.start_individuals());
        threads.extend(self.start_visibilities());
        threads.extend(self.start_squads());
        threads.push(self.start_world());
        threads.push(self.start_smoke());
        threads.extend(self.start_ais());
        threads.push(self.start_scheduler());

        let _ = ready.send(Ok(()));
        #[cfg(feature = "perfs")]
        self.perfs();

        match self.signal.1.recv() {
            Ok(Signal::End) => {
                tracing::info!("Battle ended, stop runner threads");
                self.stop(threads);
                self.end();
            }
            Ok(Signal::Stop) | Err(_) => {
                self.stopped.store(true, Ordering::Relaxed);
            }
        }

        tracing::debug!("Finished runner");
    }

    /// Stop all runner threads and wait them to finish their current tick
    fn stop(&self, threads: Vec<JoinHandle<()>>) {
        self.stopped.store(true, Ordering::Relaxed);
        for thread in threads {
            let _ = thread.join();
        }
    }

    /// Battle is ended and runner threads are stopped
    fn end(&self) {
        let ctx = self.context();
        // Destroyed vehicles stay on the map as wrecks
        self.state.world_mut().wreck_destroyed_vehicles();
        let world = self.state.world();
        let world: &oc_world::World = &world;

        #[cfg(feature = "battle_instrument")]
        crate::battle_log::summary(world, "end");

        // Persist before announcing the end: clients may exit as soon as they receive it
        match &self.end {
            EndPolicy::None => {}
            EndPolicy::SaveMapPersistTo(path) => {
                if let Err(error) = snapshot::State::from(world).save(path) {
                    tracing::error!("Save map persist: {error}");
                }
            }
        }

        // Send to each side its battle summary
        for side in [Side::A, Side::B] {
            let resume = EndResume::from_world(world, side);
            ctx.broadcast(Listening::Side(side), vec![ToClient::EndResume(resume)]);
        }
    }

    fn context(&self) -> Context<E> {
        Context::new(
            self.state.clone(),
            self.output.clone(),
            #[cfg(feature = "tracker")]
            self.tracker.clone(),
            #[cfg(feature = "debug")]
            self.direct.clone(),
            self.stopped.clone(),
            self.signal.0.clone(),
        )
    }

    /// Forward external stop request to runner main loop
    fn listen_stop(&self, stop: Receiver<()>) {
        let signal = self.signal.0.clone();
        std::thread::spawn(move || {
            // Err means stop sender dropped: no stop will ever be requested
            if stop.recv().is_ok() {
                let _ = signal.send(Signal::Stop);
            }
        });
    }

    fn start_physics(&self) -> Vec<JoinHandle<()>> {
        tracing::debug!("Start physics");
        let ctx = self.context();

        (0..ctx.cpus)
            .map(|i| {
                spawn_ticker(ctx.clone(), Kind::Physics, true, move |ctx| {
                    tracing::trace!(name = "runner-physics", i = i);
                    for update_ in physics::Processor::new(ctx).step(i) {
                        update::update(ctx, update_);
                    }
                })
            })
            .collect()
    }

    fn start_individuals(&self) -> Vec<JoinHandle<()>> {
        tracing::debug!("Start individuals");
        let ctx = self.context();
        let count = self.state.world().individuals().len();

        spawn_chunked(ctx, Kind::Individuals, count, |ctx, batch| {
            let world = &ctx.state.world();
            let indexes = &ctx.state.indexes();
            batch
                .iter()
                .flat_map(|i| {
                    let i = IndividualIndex(*i as u64);
                    tracing::trace!(name = "runner-individual", i = ?i);
                    individual::Processor::new(world, indexes, i).step()
                })
                .collect::<Vec<_>>()
        })
    }

    fn start_squads(&self) -> Vec<JoinHandle<()>> {
        tracing::debug!("Start squads");
        let ctx = self.context();
        let count = self.state.world().squads().len();

        spawn_chunked(ctx, Kind::Squads, count, |ctx, batch| {
            let world = &ctx.state.world();
            let indexes = &ctx.state.indexes();
            batch
                .iter()
                .flat_map(|i| {
                    let i = SquadIndex(*i as u64);
                    tracing::trace!(name = "runner-squad", i = ?i);
                    squad::Processor::new(world, indexes, i).step()
                })
                .collect::<Vec<_>>()
        })
    }

    fn start_visibilities(&self) -> Option<JoinHandle<()>> {
        tracing::debug!("Start visibilities");
        let ctx = self.context();
        if self.state.world().individuals().is_empty() {
            return None;
        }

        Some(spawn_ticker(ctx, Kind::Visibilities, false, |ctx| {
            let visibilities = {
                // Short lock to start: sides individuals
                let sides = {
                    let world = &ctx.state.world();
                    let indexes = &ctx.state.indexes();
                    tracing::trace!(name = "runner-visibilities");

                    matches!(world.phase, BattlePhase::Fight).then(|| {
                        (
                            indexes.side_individuals(Side::A).to_vec(),
                            indexes.side_individuals(Side::B).to_vec(),
                        )
                    })
                };
                let Some((side_a, side_b)) = sides else {
                    return;
                };

                // Then compute by chunks of watchers, releasing locks between chunks to let
                // writers (individuals, physics, ...) go through
                let mut individuals = vec![];
                let mut vehicles = vec![];
                for (watchers, targets) in [(&side_a, &side_b), (&side_b, &side_a)] {
                    for chunk in watchers.chunks(VISIBILITIES_CHUNK) {
                        let world = &ctx.state.world();
                        let indexes = &ctx.state.indexes();
                        let processor = crate::visibility::Processor::new(world, indexes);
                        individuals.extend(processor.compute_individuals_for(chunk, targets));
                        vehicles.extend(processor.compute_vehicles_for(chunk));
                    }
                }
                (individuals, vehicles)
            };

            let (individuals, vehicles) = visibilities;
            update::update(
                ctx,
                update::Update::UpdateVisibilities(individuals, vehicles),
            );
        }))
    }

    fn start_world(&self) -> JoinHandle<()> {
        tracing::debug!("Start world");

        spawn_ticker(self.context(), Kind::World, true, |ctx| {
            let updates = {
                let world = &ctx.state.world();
                tracing::trace!(name = "runner-world");

                crate::world::Processor::new(world).compute()
            };

            for update in updates {
                update::update(ctx, update);
            }
        })
    }

    fn start_ais(&self) -> Vec<JoinHandle<()>> {
        self.ais
            .iter()
            .cloned()
            .map(|config| self.start_ai(config))
            .collect()
    }

    fn start_ai(&self, config: crate::ai::AiConfig) -> JoinHandle<()> {
        tracing::info!(name="runner-start-ai", config=?config);
        let mut ai = crate::ai::Ai::from_config(config);

        spawn_ticker(self.context(), Kind::Ai, true, move |ctx| {
            let perception = ai.perceive(&ctx.state.world(), &ctx.state.indexes());
            let commands = ai.decide(&perception);
            true_!(!commands.is_empty(), return);
            let updates = ai.updates(&ctx.state.world(), commands);
            update::update_all(ctx, updates);
        })
    }

    fn start_smoke(&self) -> JoinHandle<()> {
        tracing::debug!("Start smoke");

        spawn_ticker(self.context(), Kind::Smoke, true, |ctx| {
            tracing::trace!(name = "runner-smoke");
            let has_clouds = !ctx.state.world().smoke_clouds.is_empty();
            if has_clouds {
                update::update(ctx, update::Update::UpdateSmoke);
            }
        })
    }

    #[cfg(feature = "perfs")]
    fn perfs(&self) {
        let state = self.state.clone();
        std::thread::spawn(move || {
            tracing::debug!("Track perfs");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));

                let loads = state
                    .perf
                    .take()
                    .iter()
                    .map(|(name, percent)| format!("{name}: {percent:.0}%"))
                    .collect::<Vec<String>>()
                    .join(", ");
                println!("load: {loads}");
            }
        });
    }

    fn listen_input(&self, input: Receiver<Event<E>>) {
        tracing::debug!("Listen inputs");
        let state = self.state.clone();
        let output = self.output.clone();
        let w = self.state.w.clone();
        let mod_ = self.state.world().mod_().clone();
        let meta = self.state.world().meta().clone();
        let static_ = self.config.static_.clone();
        let persist_end = self.end.persists();

        let ctx = self.context();

        std::thread::spawn(move || {
            while let Ok(message) = input.recv() {
                match message {
                    Event::Connected(endpoint) => {
                        state.listeners_mut().push(endpoint.clone());

                        let config = GameConfig::new(
                            w.clone(),
                            mod_.clone(),
                            meta.clone(),
                            static_.clone(),
                            persist_end,
                        );
                        output
                            .send((endpoint.clone(), ToClient::GameConfig(Box::new(config))))
                            .unwrap(); // TODO
                    }
                    Event::Disconnected(endpoint) => state.listeners_mut().remove(&endpoint),
                    // Battle ended: ignore clients messages which would alter the world, but keep
                    // answering others so clients can still display the final state
                    Event::Message(_, message) if ctx.stopped() && !read_only(&message) => {}
                    Event::Message(endpoint, message) => {
                        for update_ in Dealer::new(&state, &mod_, &output, endpoint).deal(message) {
                            update::update(&ctx, update_);
                        }
                    }
                }
            }
        });
    }

    fn start_scheduler(&self) -> JoinHandle<()> {
        tracing::debug!("Start scheduler");
        let ctx = self.context();

        std::thread::spawn(move || {
            while !ctx.stopped() {
                let now = Instant::now();
                // Take all tasks at once: lock must be released before run updates
                let tasks = std::mem::take(&mut *ctx.state.scheduled());
                let (due, later): (Vec<_>, Vec<_>) =
                    tasks.into_iter().partition(|(instant, _)| now >= *instant);
                ctx.state.scheduled().extend(later);
                for (_, update_) in due {
                    update::update(&ctx, update_);
                }

                // TODO: This is a very basic system of scheduler ...
                std::thread::sleep(ctx.state.w.scheduler_tick().interval());
            }
        })
    }
}

/// Client messages which don't alter the world
fn read_only(message: &ToServer) -> bool {
    matches!(
        message,
        ToServer::RequestInit(_)
            | ToServer::ListenRegion(_)
            | ToServer::ForgotRegion(_)
            | ToServer::Refresh
    )
}

fn ended<E: Client>(ctx: &Context<E>) -> bool {
    matches!(ctx.state.world().phase, BattlePhase::End(_))
}

/// Periodic runner kinds: each one has its own tick interval and perf load
#[derive(Clone, Copy)]
enum Kind {
    Physics,
    Individuals,
    Squads,
    Visibilities,
    World,
    Smoke,
    Ai,
}

impl Kind {
    fn interval<E: Client>(self, ctx: &Context<E>) -> Duration {
        let w = &ctx.state.w;
        match self {
            Kind::Physics => w.physics_tick().interval(),
            Kind::Individuals => w.individual_tick().interval(),
            Kind::Squads => w.squad_tick().interval(),
            Kind::Visibilities => w.visibilities_tick().interval(),
            Kind::World => WORLD_INTERVAL.div_f32(w.speed()),
            Kind::Smoke => SMOKE_INTERVAL.div_f32(w.speed()).max(SMOKE_MIN_INTERVAL),
            Kind::Ai => AI_INTERVAL.div_f32(w.speed()),
        }
    }

    #[cfg(feature = "perfs")]
    fn load(self, perf: &crate::perf::Perf) -> &crate::perf::Load {
        match self {
            Kind::Physics => &perf.physics,
            Kind::Individuals => &perf.individuals,
            Kind::Squads => &perf.squads,
            Kind::Visibilities => &perf.visibilities,
            Kind::World | Kind::Smoke | Kind::Ai => &perf.world,
        }
    }
}

/// Spawn a thread calling `tick` once per `kind` interval. When `skip_ended` is set, ticks are
/// skipped once the battle is ended. Thread finishes when runner is stopped.
fn spawn_ticker<E: Client>(
    ctx: Context<E>,
    kind: Kind,
    skip_ended: bool,
    mut tick: impl FnMut(&Context<E>) + Send + 'static,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut last = Instant::now();

        loop {
            let interval = kind.interval(&ctx);
            let wait = interval - last.elapsed().min(interval);
            #[cfg(feature = "perfs")]
            kind.load(&ctx.state.perf).record(wait, interval);

            std::thread::sleep(wait);
            last = Instant::now();

            if ctx.stopped() {
                break;
            }

            if skip_ended && ended(&ctx) {
                continue;
            }

            tick(&ctx);
        }
    })
}

/// Split `0..count` between one ticker thread by cpu. Each tick, `step` is called on batches of
/// indexes (locks are released between batches) and its updates applied.
fn spawn_chunked<E: Client>(
    ctx: Context<E>,
    kind: Kind,
    count: usize,
    step: impl Fn(&Context<E>, &[usize]) -> Vec<update::Update> + Clone + Send + 'static,
) -> Vec<JoinHandle<()>> {
    let size = (count as f32 / ctx.cpus as f32).ceil() as usize;
    if size == 0 {
        return vec![];
    }

    (0..count)
        .collect::<Vec<usize>>()
        .chunks(size)
        .map(|indexes| {
            let indexes = indexes.to_vec();
            let step = step.clone();

            spawn_ticker(ctx.clone(), kind, true, move |ctx| {
                for batch in indexes.chunks(STEP_BATCH) {
                    update::update_all(ctx, step(ctx, batch));
                }
            })
        })
        .collect()
}
