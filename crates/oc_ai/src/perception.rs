use glam::Vec2;
use oc_individual::{IndividualIndex, Status, order::Order, squad::SquadIndex};
use oc_root::{battle::BattlePhase, geo::WorldVec2, physics::Meters, side::Side};
use oc_utils::{d2::Angle, polygon::Polygon};
use oc_world::{
    World,
    flag::{Flag, Ownership},
    support::SupportType,
};

/// What one side knows about the battle. Only contains information a human player of this side
/// would have (enemies are only those currently seen by an individual of the side), except
/// [`FlagView::enemies`] (AI cheat).
#[derive(Debug, Clone)]
pub struct Perception {
    pub side: Side,
    /// World distance unit (pixels) for one meter
    pub pixels_per_meter: f32,
    pub phase: BattlePhase,
    /// Squads of the side which still have alive members
    pub squads: Vec<SquadView>,
    /// Enemy individuals currently seen by at least one individual of the side, and enemies seen
    /// recently (not seen anymore, with an empty [`EnemyView::seen_by`], see
    /// [`Perception::with_remembered`])
    pub enemies: Vec<EnemyView>,
    pub flags: Vec<FlagView>,
    /// Supports of the side (with their available count)
    pub supports: Vec<SupportView>,
    /// Elapsed part of the battle duration (0.0 at fight start, 1.0 at its end)
    pub progress: f32,
    /// Morale of the side: part (0.0 to 1.0) of its individuals still operational
    pub morale: f32,
    /// Center of each enemy spawn zone (where enemy comes from)
    pub enemy_spawns: Vec<WorldVec2>,
    /// Positions where a squad can be deployed (only during deployment phase, see
    /// [`Perception::with_candidates`])
    pub candidates: Vec<Candidate>,
    /// Cover positions of squads in contact with enemy or under fire, protecting them from their
    /// enemy (only during fight phase, see [`Perception::with_covers`])
    pub covers: Vec<(SquadIndex, Candidate)>,
}

/// Position where a squad leader can be placed during deployment, or can take cover during fight
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub position: WorldVec2,
    pub kind: CandidateKind,
    /// Angle of the squad line formation (along the wall for interiors)
    pub angle: Angle,
    /// Mean opacity (0.0 to 1.0) of tiles around the position
    pub cover: f32,
}

/// Kind of deployment position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateKind {
    /// On a building wall facing the enemy (individuals inside a building only see outside from
    /// its walls)
    Interior,
    /// Map place (precise position chosen by map designer)
    Place,
    /// Any other position (like high opacity tiles)
    Open,
}

/// Squad of the side
#[derive(Debug, Clone, PartialEq)]
pub struct SquadView {
    pub index: SquadIndex,
    pub position: WorldVec2,
    pub actives: u8,
    pub orders: Vec<Order>,
    /// Placed by scenario: must not be moved during deployment
    pub placed: bool,
    /// Members recently shot at (near projectiles or losses), see [`Perception::with_under_fire`]
    pub under_fire: bool,
    /// Smoke grenades owned by operational members
    pub smokes: u8,
}

/// Enemy individual currently seen by the side
#[derive(Debug, Clone, PartialEq)]
pub struct EnemyView {
    pub index: IndividualIndex,
    pub position: WorldVec2,
    /// Squads of the side with at least one member seeing this enemy (empty for a remembered
    /// enemy, at its last seen position)
    pub seen_by: Vec<SquadIndex>,
}

/// Support (artillery, etc.) of the side
#[derive(Debug, Clone, PartialEq)]
pub struct SupportView {
    pub type_: SupportType,
    /// Count of requests still possible
    pub available: u8,
    /// Radius (meters) around the requested position where shells fall
    pub radius: f32,
}

/// Flag and its current ownership
#[derive(Debug, Clone, PartialEq)]
pub struct FlagView {
    pub center: WorldVec2,
    pub ownership: Ownership,
    /// Positions of operational enemies inside the flag. AI cheats here: they are known even if
    /// not seen (used to fight for contested flags).
    pub enemies: Vec<WorldVec2>,
}

impl FlagView {
    /// True if `side` owns (or shares) this flag
    pub fn owned_by(&self, side: Side) -> bool {
        matches!(
            (&self.ownership, side),
            (Ownership::Both, _) | (Ownership::A, Side::A) | (Ownership::B, Side::B)
        )
    }

    /// True if both sides own this flag
    pub fn contested(&self) -> bool {
        self.ownership == Ownership::Both
    }

    /// True if only `side` owns this flag (not contested)
    pub fn controlled_by(&self, side: Side) -> bool {
        matches!(
            (&self.ownership, side),
            (Ownership::A, Side::A) | (Ownership::B, Side::B)
        )
    }
}

impl Perception {
    /// What `side` knows about the battle from `world`. Deployment knowledge (placed squads,
    /// candidates) must be added with [`Perception::with_placed`] and
    /// [`Perception::with_candidates`].
    pub fn from_world(world: &World, side: Side) -> Self {
        Self {
            side,
            pixels_per_meter: Meters(1.0).pixels(&world.w),
            phase: world.phase,
            squads: squads(world, side),
            enemies: enemies(world, side),
            flags: flags(world, side),
            supports: supports(world, side),
            progress: progress(world),
            morale: morale(world, side),
            enemy_spawns: enemy_spawns(world, side),
            candidates: vec![],
            covers: vec![],
        }
    }

    /// Mark squads placed by scenario (they must not be moved during deployment)
    pub fn with_placed(mut self, placed: &[SquadIndex]) -> Self {
        for squad in &mut self.squads {
            squad.placed = placed.contains(&squad.index);
        }
        self
    }

    /// Set positions where squads can be deployed
    pub fn with_candidates(mut self, candidates: Vec<Candidate>) -> Self {
        self.candidates = candidates;
        self
    }

    /// Mark squads recently shot at
    pub fn with_under_fire(mut self, under_fire: &[SquadIndex]) -> Self {
        for squad in &mut self.squads {
            squad.under_fire = under_fire.contains(&squad.index);
        }
        self
    }

    /// Add enemies seen recently but not seen anymore (at their last seen position)
    pub fn with_remembered(mut self, remembered: Vec<EnemyView>) -> Self {
        let remembered: Vec<EnemyView> = remembered
            .into_iter()
            .filter(|r| self.enemies.iter().all(|e| e.index != r.index))
            .collect();
        self.enemies.extend(remembered);
        self
    }

    /// Position of the known enemy nearest to `position`, within `meters`: seen or remembered
    /// enemies, and enemies inside flags (AI cheat, see [`FlagView::enemies`])
    pub fn known_threat(&self, position: WorldVec2, meters: f32) -> Option<WorldVec2> {
        let seen = self.enemies.iter().map(|e| e.position);
        let in_flags = self.flags.iter().flat_map(|f| f.enemies.iter().copied());
        let near = seen
            .chain(in_flags)
            .filter(|p| self.meters(position, *p) <= meters);
        self.nearest(position, near, |p| *p)
    }

    /// Where fire on a squad at `position` probably comes from: nearest seen enemy, else nearest
    /// enemy inside a flag (AI cheat, see [`FlagView::enemies`]), else enemy front
    pub fn threat(&self, position: WorldVec2) -> Option<WorldVec2> {
        let seen = self.enemies.iter().map(|e| e.position);
        let in_flags = self.flags.iter().flat_map(|f| f.enemies.iter().copied());

        self.nearest(position, seen, |p| *p)
            .or_else(|| self.nearest(position, in_flags, |p| *p))
            .or_else(|| self.enemy_front())
    }

    /// Set cover positions of squads
    pub fn with_covers(mut self, covers: Vec<(SquadIndex, Candidate)>) -> Self {
        self.covers = covers;
        self
    }

    /// True if an enemy is seen by members of `squad`
    pub fn in_contact(&self, squad: SquadIndex) -> bool {
        self.enemies.iter().any(|e| e.seen_by.contains(&squad))
    }

    /// Cover positions of `squad`
    pub fn covers_of(&self, squad: SquadIndex) -> impl Iterator<Item = &Candidate> {
        self.covers
            .iter()
            .filter(move |(s, _)| *s == squad)
            .map(|(_, c)| c)
    }

    /// Distance in meters between two positions
    pub fn meters(&self, a: WorldVec2, b: WorldVec2) -> f32 {
        Vec2::from(a).distance(b.into()) / self.pixels_per_meter
    }

    /// Item of `items` nearest to `from` (`position` gives the position of an item)
    pub fn nearest<T>(
        &self,
        from: WorldVec2,
        items: impl IntoIterator<Item = T>,
        position: impl Fn(&T) -> WorldVec2,
    ) -> Option<T> {
        items.into_iter().min_by(|a, b| {
            self.meters(from, position(a))
                .total_cmp(&self.meters(from, position(b)))
        })
    }

    /// Mean position of enemy spawn zones
    pub fn enemy_front(&self) -> Option<WorldVec2> {
        centroid(&self.enemy_spawns)
    }

    /// Engage order for given squad, only on enemies seen by its own members (squads not in
    /// contact keep their orders): keep current target while seen within `meters`, else nearest
    /// seen enemy within `meters`
    pub fn engage(&self, squad: &SquadView, meters: f32) -> Option<Order> {
        let in_range: Vec<&EnemyView> = self.enemies_in_range(squad, meters).collect();

        if let Some(Order::Engage(target)) = squad.orders.first()
            && in_range.iter().any(|e| e.index == *target)
        {
            return Some(Order::Engage(*target));
        }

        self.nearest(squad.position, in_range, |e| e.position)
            .map(|e| Order::Engage(e.index))
    }

    /// Enemies seen by members of `squad` and within `meters` of it
    fn enemies_in_range<'a>(
        &'a self,
        squad: &'a SquadView,
        meters: f32,
    ) -> impl Iterator<Item = &'a EnemyView> {
        self.enemies
            .iter()
            .filter(|e| e.seen_by.contains(&squad.index))
            .filter(move |e| self.meters(squad.position, e.position) <= meters)
    }
}

/// Squads of `side` which still have active members
fn squads(world: &World, side: Side) -> Vec<SquadView> {
    world
        .squads()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.side == side && s.actives > 0)
        .map(|(i, s)| SquadView {
            index: SquadIndex(i as u64),
            position: s.position,
            actives: s.actives,
            orders: s.orders.clone(),
            placed: false,
            under_fire: false,
            smokes: smokes(world, s),
        })
        .collect()
}

/// Smoke grenades owned by operational members of squad
fn smokes(world: &World, squad: &oc_individual::squad::Squad) -> u8 {
    squad
        .members
        .iter()
        .map(|i| world.individual(*i))
        .filter(|i| i.status.is_target())
        .flat_map(|i| i.weapons.grenades.iter())
        .filter(|g| world.mod_.grenade(**g).nature == oc_mod::grenade::GrenadeNature::Smoke)
        .count()
        .min(u8::MAX as usize) as u8
}

/// Individuals of `side` able to see, with their squad (grouped by squad)
fn watchers(world: &World, side: Side) -> Vec<(IndividualIndex, SquadIndex)> {
    let individuals = world.individuals();

    world
        .squads()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.side == side)
        .flat_map(|(squad, s)| {
            s.members
                .iter()
                .map(move |member| (*member, SquadIndex(squad as u64)))
        })
        .filter(|(member, _)| {
            individuals
                .get(member.0 as usize)
                .is_some_and(|i| i.can_lov())
        })
        .collect()
}

/// Alive enemies of `side` seen by at least one of its individuals
fn enemies(world: &World, side: Side) -> Vec<EnemyView> {
    let watchers = watchers(world, side);

    world
        .individuals()
        .iter()
        .enumerate()
        .filter(|(_, i)| i.side != side && !matches!(i.status, Status::Dead))
        .filter_map(|(i, individual)| {
            let index = IndividualIndex(i as u64);
            let seen_by = seen_by(world, &watchers, index);
            (!seen_by.is_empty()).then(|| EnemyView {
                index,
                position: WorldVec2::new(individual.position.x, individual.position.y),
                seen_by,
            })
        })
        .collect()
}

/// Squads of `watchers` (grouped by squad) with at least one member seeing `target`
fn seen_by(
    world: &World,
    watchers: &[(IndividualIndex, SquadIndex)],
    target: IndividualIndex,
) -> Vec<SquadIndex> {
    let mut squads: Vec<SquadIndex> = watchers
        .iter()
        .filter(|(watcher, _)| {
            world
                .visibilities
                .for_(*watcher)
                .get(target.0 as usize)
                .is_some_and(|v| v.visible)
        })
        .map(|(_, squad)| *squad)
        .collect();
    squads.dedup();
    squads
}

/// All flags with their current ownership and enemies of `side` inside them
fn flags(world: &World, side: Side) -> Vec<FlagView> {
    world
        .flags
        .iter()
        .map(|(flag, ownership)| FlagView {
            center: flag.center(),
            ownership: ownership.clone(),
            enemies: enemies_in_flag(world, side, flag),
        })
        .collect()
}

/// Positions of operational enemies of `side` inside `flag` (seen or not)
fn enemies_in_flag(world: &World, side: Side, flag: &Flag) -> Vec<WorldVec2> {
    world
        .individuals()
        .iter()
        .filter(|i| i.side != side && matches!(i.status, Status::Operational))
        .map(|i| WorldVec2::new(i.position.x, i.position.y))
        .filter(|position| flag.contains(*position))
        .collect()
}

/// Elapsed part (0.0 to 1.0) of the battle duration
fn progress(world: &World) -> f32 {
    let duration = world.duration.as_secs_f32();
    match (world.elapsed(), duration > 0.0) {
        (Some(elapsed), true) => (elapsed.as_secs_f32() / duration).min(1.0),
        _ => 0.0,
    }
}

/// Part (0.0 to 1.0) of `side` individuals still operational (1.0 if it has none)
fn morale(world: &World, side: Side) -> f32 {
    let individuals = world.individuals().iter().filter(|i| i.side == side);
    let total = individuals.clone().count();
    let operational = individuals
        .filter(|i| matches!(i.status, Status::Operational))
        .count();
    match total {
        0 => 1.0,
        _ => operational as f32 / total as f32,
    }
}

/// Supports of `side` known by the mod, with their available count
fn supports(world: &World, side: Side) -> Vec<SupportView> {
    world.supports[side]
        .0
        .iter()
        .filter_map(|(t, _)| {
            let support = world.mod_.support(t.name())?;
            Some(SupportView {
                type_: t.clone(),
                available: world.available_support(side, t),
                radius: support.radius.0,
            })
        })
        .collect()
}

/// Center of each spawn zone of `side` enemy
fn enemy_spawns(world: &World, side: Side) -> Vec<WorldVec2> {
    world
        .spawn_zones(side.opposite())
        .iter()
        .filter_map(|zone| centroid(zone.points()))
        .collect()
}

/// Mean of given points
pub fn centroid(points: &[WorldVec2]) -> Option<WorldVec2> {
    if points.is_empty() {
        return None;
    }
    let sum = points
        .iter()
        .fold(Vec2::ZERO, |sum, p| sum + Vec2::from(*p));
    let mean = sum / points.len() as f32;
    Some(WorldVec2::new(mean.x, mean.y))
}
