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
//! when no cover is found.
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
    engage_orders(perception, squad, engagements)
        .or_else(|| under_fire_orders(perception, squad))
        .or_else(|| help_orders(perception, squad, engagements))
}

/// Orders of squad under fire without seeing the enemy: run to the best cover nearby then defend
/// facing the threat (see [`Perception::threat`]), or defend in place when in cover or no cover
/// is found. A squad already running to cover or defending keeps its orders. None if squad is not
/// under fire (or sees the enemy).
pub fn under_fire_orders(perception: &Perception, squad: &SquadView) -> Option<Vec<Order>> {
    if !squad.under_fire || perception.in_contact(squad.index) {
        return None;
    }
    if taking_cover(squad) || defending(squad) {
        return Some(squad.orders.clone());
    }

    let threat = perception.threat(squad.position)?;
    Some(cover_orders(perception, squad, threat))
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
        return Some(squad.orders.clone());
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
        Some(cover) => vec![Order::MoveFastTo(cover.position), defend(cover.position)],
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

    match bound(perception, squad, objective, threat) {
        Some(cover) => vec![
            Order::MoveFastTo(cover.position),
            Order::Defend(Direction::from_points2d(
                cover.position.into(),
                threat.into(),
            )),
        ],
        None => vec![Order::SneakTo(crawl_step(perception, squad, objective))],
    }
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
        .filter(|c| perception.meters(squad.position, c.position) <= COVER_SEARCH_DISTANCE)
        .filter(|c| perception.meters(threat, c.position) >= COVER_MIN_ENEMY_DISTANCE)
        .filter(|c| progress(c) >= BOUND_MIN_PROGRESS)
        .max_by(|a, b| score(a).total_cmp(&score(b)))
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
    matches!(squad.orders.first(), Some(Order::SneakTo(_)))
}

/// True if squad engages an enemy
fn engaging(squad: &SquadView) -> bool {
    matches!(squad.orders.first(), Some(Order::Engage(_)))
}

/// True if squad runs to a cover position before defending (or engaging, hiding)
fn taking_cover(squad: &SquadView) -> bool {
    matches!(
        squad.orders.as_slice(),
        [
            Order::MoveFastTo(_),
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
        .filter(|c| perception.meters(squad.position, c.position) <= COVER_SEARCH_DISTANCE)
        .filter(|c| perception.meters(enemy, c.position) >= COVER_MIN_ENEMY_DISTANCE)
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
