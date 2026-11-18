//! Defensive, rule based, AI.
//!
//! Deployment phase (once): each squad is placed near a flag owned by the side (most exposed flags
//! first), preferring interiors, map places, high opacity tiles and positions on the enemy side of
//! the flag.
//!
//! Fight phase: at fight start, the nearest squad of each owned flag becomes its defender.
//! Defenders losing members under fire fall back to a cover farther from the enemy. Defenders
//! under fire from far enemies (see [`HIDE_DISTANCE`]) or from enemies they don't see hide, else
//! engage enemies they see nearby from their position, else defend facing the enemy
//! (they never leave their flag, to take cover or help other squads). Other squads attack like
//! [`crate::Offensive`] squads (and defend when no flag is left to capture). Supports are
//! requested on groups of known enemies (see [`crate::support`]).
use glam::Vec2;
use oc_individual::{order::Order, squad::SquadIndex};
use oc_root::{battle::BattlePhase, geo::WorldVec2};
use oc_utils::d2::Direction;
use rustc_hash::FxHashMap;

use crate::{
    Commander,
    combat::{self, Engagement},
    command::Command,
    offensive,
    pace::Pace,
    perception::{Candidate, CandidateKind, Perception, SquadView},
    support,
};

/// Distance (meters) from a flag under which a position protects it
pub const PROTECT_DISTANCE: f32 = 60.0;
/// Minimal distance (meters) between two deployed squads
pub const SPACING: f32 = 20.0;
/// Distance (meters) beyond which defenders under fire hide instead of returning fire: a long
/// range duel kills exposed defenders for nothing, they fire when the enemy comes closer
pub const HIDE_DISTANCE: f32 = 100.0;
/// Cover value (see [`Candidate::cover`]) under which a defender position (not interior) is poor
pub const POOR_POSITION_COVER: f32 = 0.5;

const PROXIMITY_WEIGHT: f32 = 2.0;
const INTERIOR_WEIGHT: f32 = 1.5;
const PLACE_WEIGHT: f32 = 1.0;
const COVER_WEIGHT: f32 = 1.0;
const FRONT_WEIGHT: f32 = 1.0;

/// Defensive AI (see module documentation)
#[derive(Debug, Default)]
pub struct Defensive {
    /// Squads have been deployed
    deployed: bool,
    /// Squads holding owned flags (chosen at first fight tick)
    defenders: Option<Vec<SquadIndex>>,
    /// Pace of attacking squads
    pace: Pace,
    /// Active members of defenders when they took their current position (see [`fall_back`])
    actives: FxHashMap<SquadIndex, u8>,
}

impl Commander for Defensive {
    /// Deploy squads once during deployment phase, then give fight orders
    fn tick(&mut self, perception: &Perception) -> Vec<Command> {
        match perception.phase {
            BattlePhase::Deployment if !self.deployed => self.deploy_once(perception),
            BattlePhase::Fight => self.fight(perception),
            _ => vec![],
        }
    }
}

impl Defensive {
    /// Deploy squads if deployment candidates are known (else retry at next tick)
    fn deploy_once(&mut self, perception: &Perception) -> Vec<Command> {
        if perception.candidates.is_empty() {
            return vec![];
        }

        self.deployed = true;
        deploy(perception)
    }

    /// Fight orders, choosing defenders at first call
    fn fight(&mut self, perception: &Perception) -> Vec<Command> {
        let defenders = self.defenders.get_or_insert_with(|| defenders(perception));
        let regroup = self.pace.regroup(perception);
        fight(perception, defenders, &mut self.actives, regroup)
    }
}

/// Place each squad (not placed by scenario) near a flag to protect, spaced from other squads
fn deploy(perception: &Perception) -> Vec<Command> {
    let front = perception.enemy_front();
    let anchors = anchors(perception, front);
    // Squads placed by scenario keep their position (other squads are spaced from them)
    let (placed, movable): (Vec<&SquadView>, Vec<&SquadView>) =
        perception.squads.iter().partition(|s| s.placed);
    let mut chosen: Vec<WorldVec2> = placed.iter().map(|s| s.position).collect();

    movable
        .into_iter()
        .enumerate()
        .filter_map(|(i, squad)| {
            let anchor = anchor_of(&anchors, i);
            let candidate = best_candidate(perception, anchor, front, &chosen)?;
            chosen.push(candidate.position);
            Some(Command::SetLeaderPosition(
                squad.index,
                candidate.position,
                candidate.angle,
            ))
        })
        .collect()
}

/// Position protected by the `i`th movable squad (squads are distributed over anchors)
fn anchor_of(anchors: &[WorldVec2], i: usize) -> Option<WorldVec2> {
    (!anchors.is_empty()).then(|| anchors[i % anchors.len()])
}

/// Best scored candidate spaced from `chosen` positions. When not enough room, accept closer
/// squads (but never the same position).
fn best_candidate<'a>(
    perception: &'a Perception,
    anchor: Option<WorldVec2>,
    front: Option<WorldVec2>,
    chosen: &[WorldVec2],
) -> Option<&'a Candidate> {
    let score = |c: &Candidate| score(perception, c, anchor, front);
    let best = |spacing: f32| {
        perception
            .candidates
            .iter()
            .filter(|c| is_spaced(perception, c.position, chosen, spacing))
            .max_by(|a, b| score(a).total_cmp(&score(b)))
    };

    best(SPACING).or_else(|| best(f32::EPSILON))
}

/// True if `position` is at least `spacing` meters from each of `others`
fn is_spaced(
    perception: &Perception,
    position: WorldVec2,
    others: &[WorldVec2],
    spacing: f32,
) -> bool {
    others
        .iter()
        .all(|other| perception.meters(*other, position) >= spacing)
}

/// Centers of flags owned by the side (contested ones included)
fn owned_flags(perception: &Perception) -> Vec<WorldVec2> {
    perception
        .flags
        .iter()
        .filter(|f| f.owned_by(perception.side))
        .map(|f| f.center)
        .collect()
}

/// Positions to protect: owned flags (else all flags), most exposed (nearest to enemy) first
fn anchors(perception: &Perception, front: Option<WorldVec2>) -> Vec<WorldVec2> {
    let owned = owned_flags(perception);
    let flags = if owned.is_empty() {
        perception.flags.iter().map(|f| f.center).collect()
    } else {
        owned
    };

    sort_by_front(perception, flags, front)
}

/// Positions sorted by distance to enemy front (nearest first)
fn sort_by_front(
    perception: &Perception,
    mut positions: Vec<WorldVec2>,
    front: Option<WorldVec2>,
) -> Vec<WorldVec2> {
    if let Some(front) = front {
        positions.sort_by(|a, b| {
            perception
                .meters(*a, front)
                .total_cmp(&perception.meters(*b, front))
        });
    }

    positions
}

/// Deployment score of candidate (higher is better), see the `*_score` functions
fn score(
    perception: &Perception,
    candidate: &Candidate,
    anchor: Option<WorldVec2>,
    front: Option<WorldVec2>,
) -> f32 {
    let base = kind_score(candidate) + cover_score(candidate);

    let Some(anchor) = anchor else {
        return base;
    };

    base + proximity_score(perception, candidate, anchor) + front_score(candidate, anchor, front)
}

/// Interiors are preferred to map places, which are preferred to other positions
pub(crate) fn kind_score(candidate: &Candidate) -> f32 {
    match candidate.kind {
        CandidateKind::Interior => INTERIOR_WEIGHT,
        CandidateKind::Place => PLACE_WEIGHT,
        CandidateKind::Open => 0.0,
    }
}

/// Higher when tiles around candidate are opaque
pub(crate) fn cover_score(candidate: &Candidate) -> f32 {
    candidate.cover.clamp(0.0, 1.0) * COVER_WEIGHT
}

/// 1.0 on the anchor, 0.0 at protect distance, down to -1.0 far from it (weighted)
fn proximity_score(perception: &Perception, candidate: &Candidate, anchor: WorldVec2) -> f32 {
    let distance = perception.meters(candidate.position, anchor);
    (1.0 - distance / PROTECT_DISTANCE).max(-1.0) * PROXIMITY_WEIGHT
}

/// 1.0 when candidate is between anchor and enemy, -1.0 when behind anchor (weighted)
fn front_score(candidate: &Candidate, anchor: WorldVec2, front: Option<WorldVec2>) -> f32 {
    let toward = Vec2::from(candidate.position) - Vec2::from(anchor);
    front
        .map(|front| Vec2::from(front) - Vec2::from(anchor))
        .filter(|_| toward.length() > f32::EPSILON)
        .map(|enemy| toward.normalize_or_zero().dot(enemy.normalize_or_zero()))
        .unwrap_or_default()
        * FRONT_WEIGHT
}

/// Nearest squad of each owned flag, most exposed flags first (a squad defends one flag only)
fn defenders(perception: &Perception) -> Vec<SquadIndex> {
    let mut defenders: Vec<SquadIndex> = vec![];

    for flag in sort_by_front(
        perception,
        owned_flags(perception),
        perception.enemy_front(),
    ) {
        let free = perception
            .squads
            .iter()
            .filter(|s| !defenders.contains(&s.index));
        if let Some(squad) = perception.nearest(flag, free, |s| s.position) {
            defenders.push(squad.index);
        }
    }

    tracing::debug!(name="ai-defenders", side=?perception.side, defenders=?defenders);
    defenders
}

/// Orders of all squads: defenders hold their flag, other squads attack
fn fight(
    perception: &Perception,
    defenders: &[SquadIndex],
    actives: &mut FxHashMap<SquadIndex, u8>,
    regroup: bool,
) -> Vec<Command> {
    let squads: Vec<&SquadView> = perception.squads.iter().collect();
    let engagements = combat::engagements(perception, &squads);
    let (defending, attacking): (Vec<&SquadView>, Vec<&SquadView>) = squads
        .into_iter()
        .partition(|s| defenders.contains(&s.index));

    defenders_orders(perception, &defending, &engagements, actives)
        .into_iter()
        .chain(attackers_orders(
            perception,
            &attacking,
            &engagements,
            regroup,
        ))
        .filter_map(|(squad, orders)| Command::set_orders_if_changed(squad, orders?))
        .chain(support::requests(perception))
        .collect()
}

/// Defenders losing members under fire fall back (see [`fall_back`]), defenders under fire from
/// far enemies hide (see [`HIDE_DISTANCE`]), else engage enemies they see nearby from their
/// position (they were deployed in cover), else defend their position. None to keep current
/// order. `actives` holds active members of defenders when they took their position.
fn defenders_orders<'a>(
    perception: &Perception,
    squads: &[&'a SquadView],
    engagements: &[Engagement],
    actives: &mut FxHashMap<SquadIndex, u8>,
) -> Vec<(&'a SquadView, Option<Vec<Order>>)> {
    squads
        .iter()
        .map(|squad| {
            let orders = fall_back(perception, squad, actives)
                .map(|o| ("fall_back", o))
                .or_else(|| hide(perception, squad, engagements).map(|o| ("hide", o)))
                .or_else(|| combat::engage_in_place(squad, engagements).map(|o| ("engage", o)))
                .or_else(|| defend(perception, squad).map(|order| ("defend", vec![order])));
            let orders = orders.map(|(_reason, orders)| {
                crate::instrument(perception, squad, _reason);
                orders
            });
            (*squad, orders)
        })
        .collect()
}

/// Orders of a defender under fire which lost members at its position, or which is in a poor
/// position (see [`poor_position`]): move to a cover farther from the threat (see
/// [`combat::withdraw_cover`]) and defend there. A defender moving to its
/// fall back position keeps its orders. None if defender holds its position.
fn fall_back(
    perception: &Perception,
    squad: &SquadView,
    actives: &mut FxHashMap<SquadIndex, u8>,
) -> Option<Vec<Order>> {
    if matches!(squad.orders.first(), Some(Order::MoveFastTo(_))) {
        return Some(squad.orders.clone());
    }
    let initial = *actives.entry(squad.index).or_insert(squad.actives);
    if !squad.under_fire || (squad.actives >= initial && !poor_position(perception, squad)) {
        return None;
    }
    let threat = perception.threat(squad.position)?;
    let cover = combat::withdraw_cover(perception, squad, threat)?;
    actives.insert(squad.index, squad.actives);
    Some(vec![
        Order::MoveFastTo(cover.position),
        Order::Defend(Direction::from_points2d(
            cover.position.into(),
            threat.into(),
        )),
    ])
}

/// True if squad is not near an interior or an opaque cover position (see
/// [`POOR_POSITION_COVER`])
fn poor_position(perception: &Perception, squad: &SquadView) -> bool {
    !perception.covers_of(squad.index).any(|c| {
        perception.meters(c.position, squad.position) <= combat::IN_COVER_DISTANCE
            && (c.kind == CandidateKind::Interior || c.cover >= POOR_POSITION_COVER)
    })
}

/// Hide order (facing the threat) of a defender under fire when the threat is beyond
/// [`HIDE_DISTANCE`], or when it sees no enemy to fire back at (staying exposed to unseen
/// shooters kills it for nothing). Current hide is kept. None if defender must not hide.
fn hide(
    perception: &Perception,
    squad: &SquadView,
    engagements: &[Engagement],
) -> Option<Vec<Order>> {
    if !squad.under_fire {
        return None;
    }
    let threat = perception.threat(squad.position)?;
    let engaged = engagements.iter().any(|e| e.squad == squad.index);
    if engaged && perception.meters(squad.position, threat) <= HIDE_DISTANCE {
        return None;
    }
    if matches!(squad.orders.first(), Some(Order::Hide(_))) {
        return Some(squad.orders.clone());
    }
    Some(vec![Order::Hide(Direction::from_points2d(
        squad.position.into(),
        threat.into(),
    ))])
}

/// Attackers act like offensive squads, and defend when no flag is left to capture. None to keep
/// current order.
fn attackers_orders<'a>(
    perception: &Perception,
    squads: &[&'a SquadView],
    engagements: &[Engagement],
    regroup: bool,
) -> Vec<(&'a SquadView, Option<Vec<Order>>)> {
    offensive::decide(perception, squads, engagements, regroup)
        .into_iter()
        .map(|(squad, orders)| {
            let orders = orders.or_else(|| defend(perception, squad).map(|order| vec![order]));
            (squad, orders)
        })
        .collect()
}

/// Defend order facing nearest seen enemy (else enemy front). None to keep current defense.
fn defend(perception: &Perception, squad: &SquadView) -> Option<Order> {
    if is_defending(squad) {
        return None;
    }

    defend_direction(perception, squad).map(Order::Defend)
}

/// True if squad already defends in a direction (a hiding squad must defend again when hide
/// conditions end, see [`hide`])
fn is_defending(squad: &SquadView) -> bool {
    matches!(squad.orders.first(), Some(Order::Defend(_)))
}

/// Direction from squad to nearest seen enemy, else to enemy front
fn defend_direction(perception: &Perception, squad: &SquadView) -> Option<Direction> {
    let enemies = perception.enemies.iter().map(|e| e.position);
    let target = perception
        .nearest(squad.position, enemies, |p| *p)
        .or_else(|| perception.enemy_front())?;

    Some(Direction::from_points2d(
        squad.position.into(),
        target.into(),
    ))
}

#[cfg(test)]
mod tests {
    use oc_individual::{IndividualIndex, squad::SquadIndex};
    use oc_root::{battle::BattlePhase, side::Side};
    use oc_world::flag::Ownership;

    use oc_utils::d2::Angle;

    use super::*;
    use crate::perception::{EnemyView, FlagView};

    fn squad(index: u64, x: f32) -> SquadView {
        SquadView {
            index: SquadIndex(index),
            position: WorldVec2::new(x, 0.0),
            actives: 4,
            orders: vec![],
            placed: false,
            under_fire: false,
            smokes: 0,
        }
    }

    fn candidate(x: f32, y: f32, kind: CandidateKind, cover: f32) -> Candidate {
        Candidate {
            position: WorldVec2::new(x, y),
            kind,
            angle: Angle::zero(),
            cover,
        }
    }

    /// Side B owns a flag at (0, 0), enemy comes from north (negative y)
    fn perception(phase: BattlePhase) -> Perception {
        Perception {
            side: Side::B,
            pixels_per_meter: 1.0,
            phase,
            squads: vec![squad(0, 0.0)],
            enemies: vec![],
            flags: vec![
                FlagView {
                    center: WorldVec2::new(0.0, 0.0),
                    ownership: Ownership::B,
                    enemies: vec![],
                },
                FlagView {
                    center: WorldVec2::new(500.0, 0.0),
                    ownership: Ownership::A,
                    enemies: vec![],
                },
            ],
            supports: vec![],
            progress: 0.0,
            morale: 1.0,
            enemy_spawns: vec![WorldVec2::new(0.0, -1000.0)],
            candidates: vec![],
            covers: vec![],
        }
    }

    #[test]
    fn deploy_waits_candidates() {
        let mut ai = Defensive::default();
        assert!(ai.tick(&perception(BattlePhase::Deployment)).is_empty());
        assert!(!ai.deployed);
    }

    #[test]
    fn deploy_prefers_interior_on_enemy_side_of_owned_flag() {
        let mut perception = perception(BattlePhase::Deployment);
        perception.candidates = vec![
            // Near enemy owned flag: ignored
            candidate(500.0, -10.0, CandidateKind::Interior, 1.0),
            // Behind owned flag
            candidate(0.0, 20.0, CandidateKind::Interior, 0.5),
            // Enemy side of owned flag
            candidate(0.0, -20.0, CandidateKind::Interior, 0.5),
            candidate(5.0, -20.0, CandidateKind::Open, 0.5),
        ];

        let mut ai = Defensive::default();
        let commands = ai.tick(&perception);
        assert_eq!(
            commands,
            vec![Command::SetLeaderPosition(
                SquadIndex(0),
                WorldVec2::new(0.0, -20.0),
                Angle::zero()
            )]
        );

        // Only once
        assert!(ai.tick(&perception).is_empty());
    }

    #[test]
    fn deploy_spaces_squads() {
        let mut perception = perception(BattlePhase::Deployment);
        perception.squads = vec![squad(0, 0.0), squad(1, 0.0)];
        perception.candidates = vec![
            candidate(0.0, -20.0, CandidateKind::Interior, 1.0),
            candidate(2.0, -20.0, CandidateKind::Interior, 1.0),
            candidate(30.0, -10.0, CandidateKind::Place, 0.5),
        ];

        let commands = Defensive::default().tick(&perception);
        assert_eq!(
            commands,
            vec![
                Command::SetLeaderPosition(
                    SquadIndex(0),
                    WorldVec2::new(0.0, -20.0),
                    Angle::zero()
                ),
                Command::SetLeaderPosition(
                    SquadIndex(1),
                    WorldVec2::new(30.0, -10.0),
                    Angle::zero()
                ),
            ]
        );
    }

    #[test]
    fn deploy_keeps_placed_squads() {
        let mut perception = perception(BattlePhase::Deployment);
        let mut placed = squad(0, 0.0);
        placed.position = WorldVec2::new(0.0, -20.0);
        perception.squads = vec![placed, squad(1, 0.0)];
        perception.candidates = vec![
            candidate(0.0, -20.0, CandidateKind::Interior, 1.0),
            candidate(2.0, -20.0, CandidateKind::Interior, 1.0),
            candidate(30.0, -10.0, CandidateKind::Place, 0.5),
        ];
        let perception = perception.with_placed(&[SquadIndex(0)]);

        let commands = Defensive::default().tick(&perception);
        assert_eq!(
            commands,
            vec![Command::SetLeaderPosition(
                SquadIndex(1),
                WorldVec2::new(30.0, -10.0),
                Angle::zero()
            )]
        );
    }

    #[test]
    fn fight_defends_facing_enemy_front() {
        let commands = Defensive::default().tick(&perception(BattlePhase::Fight));
        let expected = Direction::from_points2d(Vec2::ZERO, Vec2::new(0.0, -1000.0));
        assert_eq!(
            commands,
            vec![Command::SetOrders(
                SquadIndex(0),
                vec![Order::Defend(expected)]
            )]
        );
    }

    #[test]
    fn fight_defends_owned_flag_and_attacks_with_others() {
        let mut perception = perception(BattlePhase::Fight);
        perception.squads = vec![squad(0, 100.0), squad(1, 10.0)];
        let expected = Direction::from_points2d(Vec2::new(10.0, 0.0), Vec2::new(0.0, -1000.0));
        assert_eq!(
            Defensive::default().tick(&perception),
            vec![
                Command::SetOrders(SquadIndex(1), vec![Order::Defend(expected)]),
                Command::SetOrders(
                    SquadIndex(0),
                    vec![Order::MoveTo(WorldVec2::new(500.0, 0.0))]
                ),
            ]
        );
    }

    #[test]
    fn fight_keeps_defending_and_engages_near_enemy() {
        let mut perception = perception(BattlePhase::Fight);
        perception.squads[0].orders = vec![Order::Defend(Direction::from_points2d(
            Vec2::ZERO,
            Vec2::new(0.0, -1.0),
        ))];
        assert!(Defensive::default().tick(&perception).is_empty());

        perception.enemies = vec![EnemyView {
            index: IndividualIndex(9),
            position: WorldVec2::new(0.0, -100.0),
            seen_by: vec![SquadIndex(0)],
        }];
        assert_eq!(
            Defensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(0),
                vec![Order::Engage(IndividualIndex(9))]
            )]
        );
    }

    #[test]
    fn defender_hides_under_unseen_near_fire_and_engages_seen_enemy() {
        let mut perception = perception(BattlePhase::Fight);
        perception.squads[0].under_fire = true;
        let y = -(HIDE_DISTANCE - 50.0);
        perception.enemies = vec![EnemyView {
            index: IndividualIndex(9),
            position: WorldVec2::new(0.0, y),
            seen_by: vec![],
        }];
        // Near shooter, not seen: hide facing it
        let hide = Order::Hide(Direction::from_points2d(Vec2::ZERO, Vec2::new(0.0, y)));
        assert_eq!(
            Defensive::default().tick(&perception),
            vec![Command::SetOrders(SquadIndex(0), vec![hide])]
        );
        // Seen: fire back
        perception.enemies[0].seen_by = vec![SquadIndex(0)];
        assert_eq!(
            Defensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(0),
                vec![Order::Engage(IndividualIndex(9))]
            )]
        );
    }

    #[test]
    fn defender_hides_under_far_fire_then_defends_again() {
        let mut perception = perception(BattlePhase::Fight);
        perception.squads[0].under_fire = true;
        let enemy = |y: f32| EnemyView {
            index: IndividualIndex(9),
            position: WorldVec2::new(0.0, y),
            seen_by: vec![],
        };
        // Far enemy: hide facing it
        perception.enemies = vec![enemy(-(HIDE_DISTANCE + 50.0))];
        let hide = Order::Hide(Direction::from_points2d(
            Vec2::ZERO,
            Vec2::new(0.0, -(HIDE_DISTANCE + 50.0)),
        ));
        let mut defensive = Defensive::default();
        assert_eq!(
            defensive.tick(&perception),
            vec![Command::SetOrders(SquadIndex(0), vec![hide.clone()])]
        );
        // Fire stops: defend again
        perception.squads[0].orders = vec![hide];
        perception.squads[0].under_fire = false;
        assert!(matches!(
            defensive.tick(&perception).as_slice(),
            [Command::SetOrders(SquadIndex(0), orders)] if matches!(orders.as_slice(), [Order::Defend(_)])
        ));
    }

    #[test]
    fn defender_falls_back_from_poor_position_under_fire() {
        let mut perception = perception(BattlePhase::Fight);
        perception.squads[0].under_fire = true;
        perception.squads[0].orders = vec![Order::Defend(Direction::from_points2d(
            Vec2::ZERO,
            Vec2::new(0.0, -1.0),
        ))];
        // Near enemy (no hide), squad in the open
        perception.enemies = vec![EnemyView {
            index: IndividualIndex(9),
            position: WorldVec2::new(0.0, -80.0),
            seen_by: vec![],
        }];
        perception.covers = vec![
            // Current position: poor cover
            (SquadIndex(0), candidate(0.0, 0.0, CandidateKind::Open, 0.1)),
            // Nearer to the enemy: not a fall back position
            (
                SquadIndex(0),
                candidate(5.0, -20.0, CandidateKind::Interior, 1.0),
            ),
            // Farther from enemy: fall back there
            (
                SquadIndex(0),
                candidate(10.0, 25.0, CandidateKind::Interior, 0.8),
            ),
        ];
        let defend = Order::Defend(Direction::from_points2d(
            Vec2::new(10.0, 25.0),
            Vec2::new(0.0, -80.0),
        ));
        assert_eq!(
            Defensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(0),
                vec![Order::MoveFastTo(WorldVec2::new(10.0, 25.0)), defend]
            )]
        );

        // Good position: holds it
        perception.covers[0].1.kind = CandidateKind::Interior;
        assert!(!matches!(
            Defensive::default().tick(&perception).as_slice(),
            [Command::SetOrders(_, orders)] if matches!(orders.first(), Some(Order::MoveFastTo(_)))
        ));
    }
}
