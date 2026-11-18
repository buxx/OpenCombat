use std::{
    collections::HashMap,
    sync::{Arc, PoisonError},
    time::{Duration, Instant},
};

use backtrace::Backtrace;
use derive_more::Display;
use uuid::Uuid;

// NOTE: this module has been human wrote, than modified by AI (claude)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum LockKind {
    Read,
    Write,
}

pub enum RwLockEvent {
    AskForRead,
    TakeRead,
    ReleaseRead,
    AskForWrite,
    TakeWrite,
    ReleaseWrite,
}

impl RwLockEvent {
    fn kind(&self) -> LockKind {
        match self {
            RwLockEvent::AskForRead | RwLockEvent::TakeRead | RwLockEvent::ReleaseRead => {
                LockKind::Read
            }
            RwLockEvent::AskForWrite | RwLockEvent::TakeWrite | RwLockEvent::ReleaseWrite => {
                LockKind::Write
            }
        }
    }

    fn is_ask(&self) -> bool {
        matches!(self, RwLockEvent::AskForRead | RwLockEvent::AskForWrite)
    }

    fn is_take(&self) -> bool {
        matches!(self, RwLockEvent::TakeRead | RwLockEvent::TakeWrite)
    }

    fn is_release(&self) -> bool {
        matches!(self, RwLockEvent::ReleaseRead | RwLockEvent::ReleaseWrite)
    }
}

#[derive(Debug, Clone)]
pub struct LockHolder {
    pub session_uuid: Uuid,
    pub kind: LockKind,
    pub held_since: Instant,
    pub backtrace: Backtrace,
}

#[derive(Debug, Clone)]
pub struct StallReport {
    pub lock_uuid: Uuid,
    pub waiting_session: Uuid,
    pub waiting_kind: LockKind,
    pub waiting_since: Instant,
    pub waiting_backtrace: Backtrace,
    pub blockers: Vec<LockHolder>,
}

fn conflicts(waiting: LockKind, held: LockKind) -> bool {
    match waiting {
        // Write needs exclusivity: blocked by any held read or write
        LockKind::Write => true,
        // Read is only blocked by a held writer
        LockKind::Read => held == LockKind::Write,
    }
}

struct PendingEntry {
    session_uuid: Uuid,
    kind: LockKind,
    backtrace: Backtrace,
    since: Instant,
    reported: bool,
}

struct HeldEntry {
    session_uuid: Uuid,
    kind: LockKind,
    backtrace: Backtrace,
    since: Instant,
}

#[derive(Default)]
struct LockState {
    pending: Vec<PendingEntry>,
    held: Vec<HeldEntry>,
}

type Msg = (Uuid, Uuid, Backtrace, Instant, RwLockEvent);

type StallCallback = Arc<dyn Fn(StallReport) + Send + Sync>;

pub struct RwLock<T> {
    uuid: Uuid,
    inner: std::sync::RwLock<T>,
    tx: std::sync::mpsc::Sender<Msg>,
}

pub struct RwLockReadGuard<'a, T> {
    lock_uuid: Uuid,
    session_uuid: Uuid,
    inner: std::sync::RwLockReadGuard<'a, T>,
    origin: Backtrace,
    tx: std::sync::mpsc::Sender<Msg>,
}

pub struct RwLockWriteGuard<'a, T> {
    lock_uuid: Uuid,
    session_uuid: Uuid,
    inner: std::sync::RwLockWriteGuard<'a, T>,
    origin: Backtrace,
    tx: std::sync::mpsc::Sender<Msg>,
}

impl<T> RwLock<T> {
    pub fn new(value: T) -> RwLock<T> {
        Self::with_period_and_callback(value, Self::default_period(), None)
    }

    pub fn with_period(value: T, period: Duration) -> RwLock<T> {
        Self::with_period_and_callback(value, period, None)
    }

    pub fn with_callback<F>(value: T, callback: F) -> RwLock<T>
    where
        F: Fn(StallReport) + Send + Sync + 'static,
    {
        Self::with_period_and_callback(value, Self::default_period(), Some(Arc::new(callback)))
    }

    pub fn with_period_and_callback(
        value: T,
        period: Duration,
        callback: Option<StallCallback>,
    ) -> RwLock<T> {
        let lock = std::sync::RwLock::new(value);
        let (tx, rx) = std::sync::mpsc::channel::<Msg>();

        std::thread::spawn(move || {
            let mut state: HashMap<Uuid, LockState> = HashMap::new();

            loop {
                match rx.recv_timeout(period) {
                    Ok((lock_uuid, session_uuid, backtrace, instant, event)) => {
                        let lock_state = state.entry(lock_uuid).or_default();

                        if event.is_ask() {
                            lock_state.pending.push(PendingEntry {
                                session_uuid,
                                kind: event.kind(),
                                backtrace,
                                since: instant,
                                reported: false,
                            });
                        } else if event.is_take() {
                            lock_state
                                .pending
                                .retain(|p| p.session_uuid != session_uuid);
                            lock_state.held.push(HeldEntry {
                                session_uuid,
                                kind: event.kind(),
                                backtrace,
                                since: instant,
                            });
                        } else if event.is_release() {
                            lock_state.held.retain(|h| h.session_uuid != session_uuid);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }

                if let Some(callback) = &callback {
                    let now = Instant::now();
                    for (lock_uuid, lock_state) in state.iter_mut() {
                        for pending in lock_state.pending.iter_mut() {
                            if pending.reported {
                                continue;
                            }
                            if now.duration_since(pending.since) < period {
                                continue;
                            }

                            let blockers: Vec<LockHolder> = lock_state
                                .held
                                .iter()
                                .filter(|h| conflicts(pending.kind, h.kind))
                                .map(|h| LockHolder {
                                    session_uuid: h.session_uuid,
                                    kind: h.kind,
                                    held_since: h.since,
                                    backtrace: h.backtrace.clone(),
                                })
                                .collect();

                            if !blockers.is_empty() {
                                pending.reported = true;
                                callback(StallReport {
                                    lock_uuid: *lock_uuid,
                                    waiting_session: pending.session_uuid,
                                    waiting_kind: pending.kind,
                                    waiting_since: pending.since,
                                    waiting_backtrace: pending.backtrace.clone(),
                                    blockers,
                                });
                            }
                        }
                    }
                }
            }
        });

        Self {
            uuid: Uuid::new_v4(),
            inner: lock,
            tx,
        }
    }

    fn default_period() -> Duration {
        let period = std::env::var("RUST_DEBUG_RW_LOCK_PERIOD").unwrap_or("1".to_string());
        let period = period.parse::<u64>().unwrap_or(1);
        Duration::from_secs(period)
    }

    pub fn read(
        &self,
    ) -> Result<RwLockReadGuard<'_, T>, PoisonError<std::sync::RwLockReadGuard<'_, T>>> {
        let backtrace = Backtrace::new();
        let session_uuid = Uuid::new_v4();
        let _ = self.tx.send((
            self.uuid,
            session_uuid,
            backtrace.clone(),
            Instant::now(),
            RwLockEvent::AskForRead,
        ));
        let lock = self.inner.read()?;
        let _ = self.tx.send((
            self.uuid,
            session_uuid,
            backtrace.clone(),
            Instant::now(),
            RwLockEvent::TakeRead,
        ));
        Ok(RwLockReadGuard {
            lock_uuid: self.uuid,
            session_uuid,
            inner: lock,
            origin: backtrace,
            tx: self.tx.clone(),
        })
    }

    pub fn write(
        &self,
    ) -> Result<RwLockWriteGuard<'_, T>, PoisonError<std::sync::RwLockWriteGuard<'_, T>>> {
        let backtrace = Backtrace::new();
        let session_uuid = Uuid::new_v4();
        let _ = self.tx.send((
            self.uuid,
            session_uuid,
            backtrace.clone(),
            Instant::now(),
            RwLockEvent::AskForWrite,
        ));
        let lock = self.inner.write()?;
        let _ = self.tx.send((
            self.uuid,
            session_uuid,
            backtrace.clone(),
            Instant::now(),
            RwLockEvent::TakeWrite,
        ));
        Ok(RwLockWriteGuard {
            lock_uuid: self.uuid,
            session_uuid,
            inner: lock,
            origin: backtrace,
            tx: self.tx.clone(),
        })
    }
}

impl<'a, T> std::ops::Deref for RwLockReadGuard<'a, T> {
    type Target = std::sync::RwLockReadGuard<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> std::ops::DerefMut for RwLockReadGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        let _ = self.tx.send((
            self.lock_uuid,
            self.session_uuid,
            self.origin.clone(),
            Instant::now(),
            RwLockEvent::ReleaseRead,
        ));
    }
}

impl<'a, T> std::ops::Deref for RwLockWriteGuard<'a, T> {
    type Target = std::sync::RwLockWriteGuard<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> std::ops::DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        let _ = self.tx.send((
            self.lock_uuid,
            self.session_uuid,
            self.origin.clone(),
            Instant::now(),
            RwLockEvent::ReleaseWrite,
        ));
    }
}

impl StallReport {
    pub fn log(&self) -> String {
        format!(
            "Stalled {} lock since {}ms. Stalled call: \n\n {:?}\n\nBlockers:\n\n{}",
            self.waiting_kind,
            self.waiting_since.elapsed().as_millis(),
            self.waiting_backtrace,
            self.blockers
                .iter()
                .map(|b| format!("{b:?}"))
                .collect::<Vec<String>>()
                .join("\n\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn write_blocked_by_unreleased_read_is_detected() {
        let (tx, rx) = mpsc::channel::<StallReport>();
        let period = Duration::from_millis(50);

        let lock = RwLock::with_period_and_callback(
            0i32,
            period,
            Some(Arc::new(move |report: StallReport| {
                let _ = tx.send(report);
            })),
        );

        // Hold a read guard so the writer below cannot proceed.
        let read_guard = lock.read().unwrap();

        std::thread::scope(|s| {
            s.spawn(|| {
                let _write_guard = lock.write().unwrap();
            });

            let report = rx
                .recv_timeout(Duration::from_secs(2))
                .expect("expected a stall report for the blocked writer");

            assert_eq!(report.waiting_kind, LockKind::Write);
            assert!(
                report.blockers.iter().any(|b| b.kind == LockKind::Read),
                "expected a held read lock to be reported as a blocker"
            );

            // Release the read so the writer thread can finish and the
            // scope can join cleanly.
            drop(read_guard);
        });
    }

    #[test]
    fn read_blocked_by_unreleased_write_is_detected() {
        let (tx, rx) = mpsc::channel::<StallReport>();
        let period = Duration::from_millis(50);

        let lock = RwLock::with_period_and_callback(
            0i32,
            period,
            Some(Arc::new(move |report: StallReport| {
                let _ = tx.send(report);
            })),
        );

        // Hold a write guard so the reader below cannot proceed.
        let write_guard = lock.write().unwrap();

        std::thread::scope(|s| {
            s.spawn(|| {
                let _read_guard = lock.read().unwrap();
            });

            let report = rx
                .recv_timeout(Duration::from_secs(2))
                .expect("expected a stall report for the blocked reader");

            assert_eq!(report.waiting_kind, LockKind::Read);
            assert!(
                report.blockers.iter().any(|b| b.kind == LockKind::Write),
                "expected a held write lock to be reported as a blocker"
            );

            drop(write_guard);
        });
    }

    #[test]
    fn no_stall_reported_without_contention() {
        let (tx, rx) = mpsc::channel::<StallReport>();
        let period = Duration::from_millis(100);

        let lock = RwLock::with_period_and_callback(
            0i32,
            period,
            Some(Arc::new(move |report: StallReport| {
                let _ = tx.send(report);
            })),
        );

        {
            let _g = lock.write().unwrap();
        }
        {
            let _g = lock.read().unwrap();
        }

        assert!(
            rx.recv_timeout(Duration::from_millis(300)).is_err(),
            "no stall report should be emitted when there is no contention"
        );
    }

    #[test]
    fn concurrent_reads_do_not_stall() {
        let (tx, rx) = mpsc::channel::<StallReport>();
        let period = Duration::from_millis(100);

        let lock = RwLock::with_period_and_callback(
            0i32,
            period,
            Some(Arc::new(move |report: StallReport| {
                let _ = tx.send(report);
            })),
        );

        let g1 = lock.read().unwrap();

        std::thread::scope(|s| {
            s.spawn(|| {
                let _g2 = lock.read().unwrap();
                std::thread::sleep(Duration::from_millis(200));
            });

            assert!(
                rx.recv_timeout(Duration::from_millis(300)).is_err(),
                "two concurrent readers should not be reported as stalled"
            );

            drop(g1);
        });
    }
}
