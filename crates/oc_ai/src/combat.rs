//! Combat rules shared by AIs: a squad is in combat when it engages an enemy seen by its own
//! members. A squad not in cover runs to the best cover position nearby (see
//! [`Perception::covers`]), then takes position there and defends facing the enemy (members are
//! placed facing it and fire on enemies they see). A squad not in combat near a squad in combat
//! goes to help it: it moves toward the enemy engaged by this squad, until its own members see an
//! enemy (then it takes cover and defends). A squad under fire without seeing the enemy runs to
//! cover (or stays in place) and defends facing the probable threat, until fire stops. A squad
//! advancing while an enemy is known nearby (seen by the side, recently seen or inside a flag)
//! moves from cover
//! to cover toward its objective, defending facing the enemy in each cover, or crawls toward it
//! when no cover is found. Exposed moves near the enemy are screened with smoke (see
//! [`smoke_screen`]). Squads keep [`SQUAD_SPACING`] between them (see [`crowded`]).
use oc_individual::{IndividualIndex, order::Order, squad::SquadIndex};
use oc_root::geo::WorldVec2;
use oc_utils::d2::Direction;

use crate::{
    defensive,
    perception::{Candidate, Perception, SquadView},
};

/// Distance (meters) under which a squad engages an enemy seen by its members
pub const ENGAGE_DISTANCE: f32 = 150.0;
/// Distance (meters) under which a squad not in combat goes to help a squad in combat
pub const HELP_DISTANCE: f32 = 100.0;
/// Distance (meters) under which a helping squad keeps its current move (avoid path changes each
/// time the followed enemy moves a little)
pub const HELP_MOVE_TOLERANCE: f32 = 10.0;
/// Distance (meters) under which a squad in contact searches a cover position
pub const COVER_SEARCH_DISTANCE: f32 = 30.0;
/// Distance (meters) from a cover position under which a squad is in cover
pub const IN_COVER_DISTANCE: f32 = 3.0;
/// Minimal distance (meters) between a cover position and the engaged enemy
pub const COVER_MIN_ENEMY_DISTANCE: f32 = 20.0;

/// Distance (meters) under which a known enemy makes squads advance from cover to cover
pub const THREAT_DISTANCE: f32 = 200.0;
/// Minimal progress (meters) toward its objective of a squad moving to the next cover
pub const BOUND_MIN_PROGRESS: f32 = 5.0;
/// Distance (meters) a squad crawls toward its objective when no cover is found
pub const CRAWL_STEP: f32 = 15.0;
/// Distance (meters) to the enemy under which a squad crawls to its next cover instead of
/// running (standing up under effective fire is deadly)
pub const CRAWL_DISTANCE: f32 = 100.0;
/// Distance (meters) under which a squad near the enemy (see [`CRAWL_DISTANCE`]) searches a cover
/// position: crawling farther exposes it too long without firing (it rather fires from its
/// position)
pub const CRAWL_COVER_SEARCH_DISTANCE: f32 = 12.0;
/// Distance (meters) under which squads advancing under threat coordinate their moves (bounding
/// overwatch, see [`may_bound`])
pub const OVERWATCH_GROUP_DISTANCE: f32 = 120.0;
/// Distance (meters) under which an overwatching squad suppresses the threat (else it only
/// defends facing it)
pub const SUPPRESS_DISTANCE: f32 = 150.0;
/// Distance (meters) under which an overwatching squad keeps suppressing its current position
/// (avoid order changes each time the threat moves a little)
pub const SUPPRESS_TOLERANCE: f32 = 15.0;
/// Minimal distance (meters) between a squad destination (cover, crawl step) and other squads
/// of the side (their position or destination): grouped squads die together (grenades, mortars,
/// machine guns)
pub const SQUAD_SPACING: f32 = 30.0;
/// Distance (meters) to the nearest known enemy under which a squad in contact can't disengage
/// (regroup, see [`crate::pace`]): it keeps fighting
pub const CLOSE_COMBAT_DISTANCE: f32 = 50.0;
/// Distance (meters) under which a squad withdrawing from the fight searches a cover position
pub const WITHDRAW_SEARCH_DISTANCE: f32 = 40.0;
/// Minimal distance gain (meters) from the threat of a withdraw cover position
pub const WITHDRAW_MIN_GAIN: f32 = 10.0;
/// Distance (meters) to the enemy under which an advancing squad owning smoke grenades screens
/// its exposed moves with smoke
pub const SMOKE_THREAT_DISTANCE: f32 = 150.0;
/// Cover value (see [`Candidate::cover`]) under which a destination is exposed (smoke needed)
pub const SMOKE_EXPOSED_COVER: f32 = 0.5;
/// Distance (meters) from squad, toward the enemy, of the smoke screen
pub const SMOKE_SCREEN_DISTANCE: f32 = 20.0;

/// Squad in combat
#[derive(Debug, Clone, PartialEq)]
pub struct Engagement {
    pub squad: SquadIndex,
    pub position: WorldVec2,
    /// Engaged enemy individual
    pub target: IndividualIndex,
}

/// Squads, among given ones, in combat
pub fn engagements(perception: &Perception, squads: &[&SquadView]) -> Vec<Engagement> {
    squads
        .iter()
        .filter_map(|squad| engagement(perception, squad))
        .collect()
}

/// Engagement of squad if it sees an enemy within [`ENGAGE_DISTANCE`]
fn engagement(perception: &Perception, squad: &SquadView) -> Option<Engagement> {
    match perception.engage(squad, ENGAGE_DISTANCE)? {
        Order::Engage(target) => Some(Engagement {
            squad: squad.index,
            position: squad.position,
            target,
        }),
        _ => None,
    }
}

/// Combat orders of squad: take cover and engage when in combat, else help the nearest squad in
/// combat. None if squad is not concerned by combat.
pub fn orders(
    perception: &Perception,
    squad: &SquadView,
    engagements: &[Engagement],
) -> Option<Vec<Order>> {
    let orders = engage_orders(perception, squad, engagements).map(|o| ("engage", o));
    let orders = orders.or_else(|| under_fire_orders(perception, squad).map(|o| ("under_fire", o)));
    let orders =
        orders.or_else(|| help_orders(perception, squad, engagements).map(|o| ("help", o)));
    let (_reason, orders) = orders?;
    crate::instrument(perception, squad, _reason);
    Some(orders)
}

/// Orders of squad under fire without seeing the enemy: run to the best cover nearby then defend
/// facing the threat (see [`Perception::threat`]), or defend in place when in cover or no cover
/// is found. A squad already running to cover or defending keeps its orders. None if squad is not
/// under fire (or sees the enemy).
pub fn under_fire_orders(perception: &Perception, squad: &SquadView) -> Option<Vec<Order>> {
    // Squads seeing an enemy within engage distance are handled by [`engage_orders`]. A squad
    // under fire from a seen enemy beyond this distance takes cover too (instead of continuing
    // to advance in the open).
    if !squad.under_fire {
        return None;
    }
    if taking_cover(squad) || defending(squad) {
        return Some(unhide(&squad.orders));
    }

    let threat = perception.threat(squad.position)?;
    Some(cover_orders(perception, squad, threat))
}

/// Orders with hide orders (kept from a regroup, see [`crate::pace`]) replaced by defend orders
/// in the same direction: a squad in combat fires back from its position, it does not stay heads
/// down (hiding individuals only fire on very near enemies)
fn unhide(orders: &[Order]) -> Vec<Order> {
    orders
        .iter()
        .map(|order| match order {
            Order::Hide(direction) => Order::Defend(*direction),
            order => order.clone(),
        })
        .collect()
}

/// True if squad defends (or hides) in a direction
fn defending(squad: &SquadView) -> bool {
    matches!(
        squad.orders.first(),
        Some(Order::Defend(_) | Order::Hide(_))
    )
}

/// Orders of squad when it is in combat. At contact: run to the best cover nearby then take
/// position and defend facing the enemy, or defend from its position when in cover. A squad
/// already running to cover or defending keeps its orders (no cover change during the fight). A
/// squad finding no cover engages the enemy from its position.
pub fn engage_orders(
    perception: &Perception,
    squad: &SquadView,
    engagements: &[Engagement],
) -> Option<Vec<Order>> {
    let engagement = engagements.iter().find(|e| e.squad == squad.index)?;

    if taking_cover(squad) || defending(squad) {
        return Some(unhide(&squad.orders));
    }
    if engaging(squad) {
        return Some(vec![Order::Engage(engagement.target)]);
    }

    let enemy = target_position(perception, engagement);
    match in_cover(perception, squad) || cover(perception, squad, enemy).is_some() {
        true => Some(cover_orders(perception, squad, enemy)),
        false => Some(vec![Order::Engage(engagement.target)]),
    }
}

/// Orders to take cover against `enemy`: defend facing it when squad is in cover (or no cover is
/// found), else run to the best cover nearby then defend facing it from there
fn cover_orders(perception: &Perception, squad: &SquadView, enemy: WorldVec2) -> Vec<Order> {
    let defend =
        |from: WorldVec2| Order::Defend(Direction::from_points2d(from.into(), enemy.into()));

    if in_cover(perception, squad) {
        return vec![defend(squad.position)];
    }

    match cover(perception, squad, enemy) {
        Some(cover) => vec![
            move_to_cover(perception, cover.position, enemy),
            defend(cover.position),
        ],
        None => vec![defend(squad.position)],
    }
}

/// Orders of squad when it is in combat: engage from its position (no cover search)
pub fn engage_in_place(squad: &SquadView, engagements: &[Engagement]) -> Option<Vec<Order>> {
    engagements
        .iter()
        .find(|e| e.squad == squad.index)
        .map(|e| vec![Order::Engage(e.target)])
}

/// Orders to advance toward `objective`. With no known enemy within [`THREAT_DISTANCE`]: move to
/// it (see [`move_to`]). Else, advance from cover to cover: run to the best cover nearer to the
/// objective (see [`bound`]) then defend facing the enemy, or crawl [`CRAWL_STEP`] meters toward
/// the objective when no cover is found. A squad running to cover or crawling keeps its orders.
pub fn advance_orders(
    perception: &Perception,
    squad: &SquadView,
    objective: WorldVec2,
    tolerance: f32,
) -> Vec<Order> {
    let threat = perception.known_threat(squad.position, THREAT_DISTANCE);
    let near = perception.meters(squad.position, objective) <= BOUND_MIN_PROGRESS;
    let Some(threat) = threat.filter(|_| !near) else {
        return vec![move_to(perception, squad, objective, tolerance)];
    };
    if taking_cover(squad) || crawling(squad) {
        return squad.orders.clone();
    }

    let (destination, exposed, orders) = match bound(perception, squad, objective, threat) {
        Some(cover) => (
            cover.position,
            cover.cover < SMOKE_EXPOSED_COVER,
            vec![
                move_to_cover(perception, cover.position, threat),
                Order::Defend(Direction::from_points2d(
                    cover.position.into(),
                    threat.into(),
                )),
            ],
        ),
        None => {
            let step = crawl_step(perception, squad, objective);
            (step, true, vec![Order::SneakTo(step)])
        }
    };

    match exposed && squad.smokes > 0 {
        true => match smoke_screen(perception, squad, destination, threat) {
            Some(smoke) => [vec![smoke], orders].concat(),
            None => orders,
        },
        false => orders,
    }
}

/// Smoke order screening the move of squad to `destination` from `threat` view: smoke between
/// them, [`SMOKE_SCREEN_DISTANCE`] meters from squad toward the threat. None if threat is too far
/// (no need) or too near (smoke would be thrown on it).
pub fn smoke_screen(
    perception: &Perception,
    squad: &SquadView,
    destination: WorldVec2,
    threat: WorldVec2,
) -> Option<Order> {
    let distance = perception.meters(squad.position, threat);
    if distance > SMOKE_THREAT_DISTANCE || distance < SMOKE_SCREEN_DISTANCE * 2.0 {
        return None;
    }
    // Screen the middle of the move
    let from = glam::Vec2::from(squad.position);
    let middle = (from + glam::Vec2::from(destination)) / 2.0;
    let toward = (glam::Vec2::from(threat) - middle).normalize_or_zero();
    let screen = middle + toward * SMOKE_SCREEN_DISTANCE * perception.pixels_per_meter;
    Some(Order::Smoke(WorldVec2::new(screen.x, screen.y)))
}

/// True if `position` is within [`SQUAD_SPACING`] of another squad of the side (its position or
/// its current move destination)
pub fn crowded(perception: &Perception, squad: &SquadView, position: WorldVec2) -> bool {
    perception
        .squads
        .iter()
        .filter(|other| other.index != squad.index)
        .any(|other| {
            let destination = other.orders.iter().find_map(|o| match o {
                Order::MoveTo(p) | Order::MoveFastTo(p) | Order::SneakTo(p) => Some(*p),
                _ => None,
            });
            std::iter::once(other.position)
                .chain(destination)
                .any(|p| perception.meters(p, position) < SQUAD_SPACING)
        })
}

/// Best cover of squad (within [`COVER_SEARCH_DISTANCE`]) at least [`BOUND_MIN_PROGRESS`] meters
/// nearer to `objective` than squad, not nearer than [`COVER_MIN_ENEMY_DISTANCE`] of `threat`
/// (higher for interiors, opaque surroundings and progress)
fn bound<'a>(
    perception: &'a Perception,
    squad: &SquadView,
    objective: WorldVec2,
    threat: WorldVec2,
) -> Option<&'a Candidate> {
    let remaining = perception.meters(squad.position, objective);
    let progress = |c: &Candidate| remaining - perception.meters(c.position, objective);
    let score = |c: &Candidate| {
        defensive::kind_score(c) + defensive::cover_score(c) + progress(c) / COVER_SEARCH_DISTANCE
    };

    perception
        .covers_of(squad.index)
        .filter(|c| {
            perception.meters(squad.position, c.position)
                <= search_distance(perception, squad, threat)
        })
        .filter(|c| perception.meters(threat, c.position) >= COVER_MIN_ENEMY_DISTANCE)
        .filter(|c| progress(c) >= BOUND_MIN_PROGRESS)
        .filter(|c| !crowded(perception, squad, c.position))
        .max_by(|a, b| score(a).total_cmp(&score(b)))
}

/// Known threat making squad advance from cover to cover toward `objective` (see
/// [`advance_orders`]), None if squad can move freely
pub fn bounding_threat(
    perception: &Perception,
    squad: &SquadView,
    objective: WorldVec2,
) -> Option<WorldVec2> {
    let near = perception.meters(squad.position, objective) <= BOUND_MIN_PROGRESS;
    perception
        .known_threat(squad.position, THREAT_DISTANCE)
        .filter(|_| !near)
}

/// True if squad is moving (running or crawling) during an advance under threat
pub fn bounding(squad: &SquadView) -> bool {
    taking_cover(squad) || crawling(squad)
}

/// Bounding overwatch: among squads advancing under threat (`bounding`: squad, remaining
/// distance to its objective) within [`OVERWATCH_GROUP_DISTANCE`] of each other, half of them
/// move while the other half covers them (see [`overwatch_orders`]). A moving squad finishes its
/// move. Rear-most squads (farthest from their objective) move first: squads leapfrog.
pub fn may_bound(
    perception: &Perception,
    squad: &SquadView,
    remaining: f32,
    bounding_: &[(&SquadView, f32)],
) -> bool {
    if bounding(squad) {
        return true;
    }
    let others: Vec<&(&SquadView, f32)> = bounding_
        .iter()
        .filter(|(other, _)| other.index != squad.index)
        .filter(|(other, _)| {
            perception.meters(squad.position, other.position) <= OVERWATCH_GROUP_DISTANCE
        })
        .collect();
    // Squads of the group (including this one) allowed to move at the same time
    let movers = others.len().div_ceil(2).max(1);
    let moving = others.iter().filter(|(other, _)| bounding(other)).count();
    if moving >= movers {
        return false;
    }
    // Waiting squads behind this one move first
    let behind = others
        .iter()
        .filter(|(other, _)| !bounding(other))
        .filter(|(other, other_remaining)| {
            *other_remaining > remaining
                || (*other_remaining == remaining && other.index.0 < squad.index.0)
        })
        .count();
    behind < movers - moving
}

/// Orders of a squad covering the advance of other squads: suppress `threat` when within
/// [`SUPPRESS_DISTANCE`], else defend facing it (from its current position)
pub fn overwatch_orders(
    perception: &Perception,
    squad: &SquadView,
    threat: WorldVec2,
) -> Vec<Order> {
    if perception.meters(squad.position, threat) > SUPPRESS_DISTANCE {
        if defending(squad) {
            return squad.orders.clone();
        }
        return vec![Order::Defend(Direction::from_points2d(
            squad.position.into(),
            threat.into(),
        ))];
    }
    match squad.orders.first() {
        Some(Order::Suppress(current))
            if perception.meters(*current, threat) <= SUPPRESS_TOLERANCE =>
        {
            vec![Order::Suppress(*current)]
        }
        _ => vec![Order::Suppress(threat)],
    }
}

/// Distance under which squad searches a cover position against `enemy`: shorter when the enemy
/// is near (squad must crawl to it, see [`CRAWL_COVER_SEARCH_DISTANCE`])
fn search_distance(perception: &Perception, squad: &SquadView, enemy: WorldVec2) -> f32 {
    match perception.meters(squad.position, enemy) <= CRAWL_DISTANCE {
        true => CRAWL_COVER_SEARCH_DISTANCE,
        false => COVER_SEARCH_DISTANCE,
    }
}

/// Move order to a cover position: crawl when the enemy is within [`CRAWL_DISTANCE`] of it, else
/// run
fn move_to_cover(perception: &Perception, cover: WorldVec2, enemy: WorldVec2) -> Order {
    match perception.meters(cover, enemy) <= CRAWL_DISTANCE {
        true => Order::SneakTo(cover),
        false => Order::MoveFastTo(cover),
    }
}

/// Position [`CRAWL_STEP`] meters from squad toward `objective` (objective if nearer)
fn crawl_step(perception: &Perception, squad: &SquadView, objective: WorldVec2) -> WorldVec2 {
    let from = glam::Vec2::from(squad.position);
    let to = glam::Vec2::from(objective);
    let step = CRAWL_STEP * perception.pixels_per_meter;
    match from.distance(to) <= step {
        true => objective,
        false => {
            let p = from + (to - from).normalize() * step;
            WorldVec2::new(p.x, p.y)
        }
    }
}

/// True if squad crawls
fn crawling(squad: &SquadView) -> bool {
    matches!(moves(squad), [Order::SneakTo(_), ..])
}

/// Squad orders, without the leading smoke screen order (see [`smoke_screen`])
fn moves(squad: &SquadView) -> &[Order] {
    match squad.orders.as_slice() {
        [Order::Smoke(_), rest @ ..] => rest,
        orders => orders,
    }
}

/// True if squad engages an enemy
fn engaging(squad: &SquadView) -> bool {
    matches!(squad.orders.first(), Some(Order::Engage(_)))
}

/// True if squad runs (or crawls) to a cover position before defending (or engaging, hiding)
fn taking_cover(squad: &SquadView) -> bool {
    matches!(
        moves(squad),
        [
            Order::MoveFastTo(_) | Order::SneakTo(_),
            Order::Defend(_) | Order::Engage(_) | Order::Hide(_),
            ..
        ]
    )
}

/// True if squad is near a cover position
pub fn in_cover(perception: &Perception, squad: &SquadView) -> bool {
    perception
        .covers_of(squad.index)
        .any(|c| perception.meters(c.position, squad.position) <= IN_COVER_DISTANCE)
}

/// Best cover position within [`COVER_SEARCH_DISTANCE`] of squad, not nearer than
/// [`COVER_MIN_ENEMY_DISTANCE`] of `enemy` (see [`cover_score`])
fn cover<'a>(
    perception: &'a Perception,
    squad: &SquadView,
    enemy: WorldVec2,
) -> Option<&'a Candidate> {
    let score = |c: &Candidate| cover_score(perception, c, squad, enemy);

    perception
        .covers_of(squad.index)
        .filter(|c| {
            perception.meters(squad.position, c.position)
                <= search_distance(perception, squad, enemy)
        })
        .filter(|c| perception.meters(enemy, c.position) >= COVER_MIN_ENEMY_DISTANCE)
        .filter(|c| !crowded(perception, squad, c.position))
        .max_by(|a, b| score(a).total_cmp(&score(b)))
}

/// Higher for interiors and opaque surroundings, lower when far from squad or nearer to enemy
/// (each distance penalty is 1.0 at [`COVER_SEARCH_DISTANCE`])
fn cover_score(
    perception: &Perception,
    candidate: &Candidate,
    squad: &SquadView,
    enemy: WorldVec2,
) -> f32 {
    let distance = perception.meters(squad.position, candidate.position);
    let approach =
        perception.meters(squad.position, enemy) - perception.meters(candidate.position, enemy);

    defensive::kind_score(candidate) + defensive::cover_score(candidate)
        - distance / COVER_SEARCH_DISTANCE
        - approach.max(0.0) / COVER_SEARCH_DISTANCE
}

/// Orders to help the nearest other squad in combat within [`HELP_DISTANCE`]: advance toward the
/// enemy it engages (see [`advance_orders`])
fn help_orders(
    perception: &Perception,
    squad: &SquadView,
    engagements: &[Engagement],
) -> Option<Vec<Order>> {
    let helped = helped(perception, squad, engagements)?;
    let target = target_position(perception, helped);

    Some(advance_orders(
        perception,
        squad,
        target,
        HELP_MOVE_TOLERANCE,
    ))
}

/// Position of the enemy engaged by squad in combat (its own position if enemy is not seen
/// anymore)
fn target_position(perception: &Perception, engagement: &Engagement) -> WorldVec2 {
    perception
        .enemies
        .iter()
        .find(|e| e.index == engagement.target)
        .map(|e| e.position)
        .unwrap_or(engagement.position)
}

/// Nearest other squad in combat within [`HELP_DISTANCE`] of squad
fn helped<'a>(
    perception: &Perception,
    squad: &SquadView,
    engagements: &'a [Engagement],
) -> Option<&'a Engagement> {
    let near = engagements.iter().filter(|e| {
        e.squad != squad.index && perception.meters(squad.position, e.position) <= HELP_DISTANCE
    });

    perception.nearest(squad.position, near, |e| e.position)
}

/// Move order to `position`. Current move is kept when it already goes within `tolerance` meters
/// of `position` (avoid path changes each time a followed position moves a little).
pub fn move_to(
    perception: &Perception,
    squad: &SquadView,
    position: WorldVec2,
    tolerance: f32,
) -> Order {
    match squad.orders.first() {
        Some(Order::MoveTo(current)) if perception.meters(*current, position) <= tolerance => {
            Order::MoveTo(*current)
        }
        _ => Order::MoveTo(position),
    }
}

/// Cover position to withdraw squad from the fight against `threat`: best cover (see
/// [`cover_score`] without approach penalty) within [`WITHDRAW_SEARCH_DISTANCE`] of squad, at
/// least [`WITHDRAW_MIN_GAIN`] meters farther from the threat, not crowded. None if no such cover.
pub fn withdraw_cover<'a>(
    perception: &'a Perception,
    squad: &SquadView,
    threat: WorldVec2,
) -> Option<&'a Candidate> {
    let current = perception.meters(squad.position, threat);
    let score = |c: &Candidate| {
        defensive::kind_score(c) + defensive::cover_score(c)
            - perception.meters(squad.position, c.position) / WITHDRAW_SEARCH_DISTANCE
    };

    perception
        .covers_of(squad.index)
        .filter(|c| perception.meters(squad.position, c.position) <= WITHDRAW_SEARCH_DISTANCE)
        .filter(|c| perception.meters(threat, c.position) >= current + WITHDRAW_MIN_GAIN)
        .filter(|c| !crowded(perception, squad, c.position))
        .max_by(|a, b| score(a).total_cmp(&score(b)))
}
