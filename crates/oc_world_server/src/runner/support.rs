use std::time::{Duration, Instant};

use crate::routing::Listening;
use crate::runner::update::{ExplosionOrigin, Update};
use crate::utils::d2::random_position_around;
use oc_network::ToClient;
use oc_root::Client;
use oc_root::geo::WorldVec2;
use oc_world::support::PendingSupport;

impl<E: Client> super::State<E> {
    pub fn add_pending_support(&self, pending: PendingSupport) -> Vec<(Listening, Vec<ToClient>)> {
        let (side, type_) = (pending.side, pending.type_.clone());
        let mut world = self.world_mut();
        let existing = world
            .pending_supports
            .iter_mut()
            .find(|p| p.side == side && p.type_ == type_);
        match existing {
            // Moved: preparation restarts (only if the fight already started it)
            Some(existing) => {
                existing.position = pending.position;
                existing.delay = pending.delay;
                existing.start = existing.start.map(|_| Instant::now() + pending.delay);
            }
            None => world.pending_supports.push(pending),
        }

        // Count takine into account the pending support just added
        let available = world.available_support(side, &type_);

        vec![(
            Listening::Side(side),
            vec![ToClient::SetSupportCount(type_, available)],
        )]
    }

    pub fn start_pending_support(
        &self,
        pending: PendingSupport,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let mut world = self.world_mut();

        if let Some(pending) = world
            .pending_supports
            .iter_mut()
            .find(|p| !p.is_started() && p.same_request(&pending))
        {
            pending.start();
        }

        vec![]
    }

    pub fn realize_support(&self, pending: PendingSupport) -> Vec<(Listening, Vec<ToClient>)> {
        if !self.consume_support(&pending) {
            // No matched pending support, abort
            return vec![];
        }

        tracing::debug!(name="runner-update-realize-support", side=?pending.side, type_=?pending.type_);
        self.realize_bombard(&pending);

        vec![]
    }

    /// Reduce the counter of the pending support type. Returns false if there is no.
    fn consume_support(&self, pending: &PendingSupport) -> bool {
        let (side, type_) = (pending.side, &pending.type_);
        let mut world = self.world_mut();

        if !world
            .pending_supports
            .iter()
            .any(|p| p.same_request(pending))
        {
            return false;
        }

        world.supports[side].consume(type_);
        // Keep in memory last realized instant (to respect delay between supports of same type)
        world
            .support_realized
            .insert((side, type_.clone()), Instant::now());

        true
    }

    fn realize_bombard(&self, pending: &PendingSupport) {
        let world = self.world();
        let mod_ = world.mod_();
        let Some(bombard) = mod_.support(pending.type_.name()) else {
            tracing::error!(name="runner-update-realize-support-unknown", type_=%pending.type_);
            return;
        };
        let target = pending.position;
        let shell = mod_.shell(bombard.shell);
        let speed = world.w.speed();

        let series: Vec<Vec<WorldVec2>> = (0..bombard.series)
            .map(|_| {
                (0..bombard.shells_by_serie)
                    .map(|_| random_position_around(&world.w, target, bombard.radius))
                    .collect()
            })
            .collect();

        let mut decal = Duration::ZERO;
        for serie in series {
            for position in serie {
                let instant = Instant::now() + decal;
                let explosion = shell.inner().clone().into();
                decal += bombard.shell_interval().div_f32(speed);
                tracing::debug!(name="runner-update-realize-support-bombard-spawn-explosion", explosion=?explosion, position=?position, decal=?decal);
                self.schedule(
                    instant,
                    Update::SpawnExplosion(
                        explosion,
                        position,
                        ExplosionOrigin::Shell(shell.index()),
                    ),
                );
            }
            decal += bombard.serie_interval().div_f32(speed);
        }
    }

    pub fn remove_pending_support(
        &self,
        pending: PendingSupport,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let mut world = self.world_mut();

        let Some(i) = world
            .pending_supports
            .iter()
            .position(|p| p.same_request(&pending))
        else {
            return vec![];
        };
        world.pending_supports.remove(i);

        vec![(
            Listening::Side(pending.side),
            vec![ToClient::SupportExecuted(pending.type_)],
        )]
    }
}
