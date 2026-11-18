use std::{
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    time::Duration,
};

/// Load of one runner: time its threads slept vs. time they were given (tick intervals)
#[derive(Debug, Default)]
pub struct Load {
    slept_us: AtomicU64,
    given_us: AtomicU64,
    /// Last percent (f32 bits), kept when no tick happened since last report
    last: AtomicU32,
}

impl Load {
    /// A thread slept `wait` of its tick `interval`
    pub fn record(&self, wait: Duration, interval: Duration) {
        self.slept_us
            .fetch_add(wait.as_micros() as u64, Ordering::Relaxed);
        self.given_us
            .fetch_add(interval.as_micros() as u64, Ordering::Relaxed);
    }

    /// Percent of load since last call: sleeping all the time is 0%, never sleeping is 100%
    fn take(&self) -> f32 {
        let slept = self.slept_us.swap(0, Ordering::Relaxed);
        let given = self.given_us.swap(0, Ordering::Relaxed);
        if given > 0 {
            let percent = (1. - slept as f32 / given as f32).clamp(0., 1.) * 100.;
            self.last.store(percent.to_bits(), Ordering::Relaxed);
        }
        f32::from_bits(self.last.load(Ordering::Relaxed))
    }
}

#[derive(Debug, Default)]
pub struct Perf {
    pub individuals: Load,
    pub squads: Load,
    pub visibilities: Load,
    pub physics: Load,
    pub world: Load,
}

impl Perf {
    /// Load percents by runner since last call
    pub fn take(&self) -> [(&'static str, f32); 5] {
        [
            ("individuals", self.individuals.take()),
            ("squads", self.squads.take()),
            ("visibilities", self.visibilities.take()),
            ("physics", self.physics.take()),
            ("world", self.world.take()),
        ]
    }
}
