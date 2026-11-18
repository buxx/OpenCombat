//! Offensive, rule based, AI. For each squad, during fight:
//!
//! - if it sees an enemy nearby: take cover nearby, then engage it (see [`crate::combat`])
//! - else if a squad in combat is near: help it
//! - else: move to a flag not controlled by the side (neutral, enemy or contested), squads being
//!   spread over these flags. For a contested flag: move to the nearest enemy inside it (see
//!   [`crate::capture`]). When an enemy is known nearby, advance from cover to cover (see
//!   [`combat::advance_orders`])
//! - else (all flags controlled): stay idle
//!
//! Squads advancing under threat near each other use bounding overwatch: one moves while the
//! others suppress the threat (see [`combat::may_bound`]). Near the enemy, squads crawl to near
//! covers instead of running. Attacks come in waves (see [`crate::pace`]): after a costly wave,
//! squads hide a while (regroup), then attack again. Squads in contact disengage (unless the
//! enemy is close), withdrawing to a cover farther from the enemy when exposed.
//!
//! Supports are requested on groups of known enemies (see [`crate::support`]).
//!
//! Orders are only given when they change, to let squads accomplish them.
use oc_individual::order::Order;
use oc_root::geo::WorldVec2;
use oc_utils::d2::Direction;

use crate::{
    Commander, capture,
    combat::{self, Engagement},
    command::Command,
    pace::Pace,
    perception::{Perception, SquadView},
    support,
};

/// Distance (meters) under which a squad keeps its current capture move (avoid path changes
/// when followed enemy moves a little)
pub const CAPTURE_MOVE_TOLERANCE: f32 = 10.0;

/// Orders of a squad holding its position (regroup): hide facing the nearest known enemy (else
/// enemy front), heads down, out of the firefight until the next wave (hiding individuals still
/// fire on near enemies). A squad still moving to a cover finishes its move then hides there. A
/// squad under fire out of cover first withdraws to a cover farther from the enemy (see
/// [`combat::withdraw_cover`]).
fn hold_orders(perception: &Perception, squad: &SquadView) -> Option<Vec<Order>> {
    let enemies = perception.enemies.iter().map(|e| e.position);
    let target = perception
        .nearest(squad.position, enemies, |p| *p)
        .or_else(|| perception.enemy_front())?;
    let hide = |from: WorldVec2| Order::Hide(Direction::from_points2d(from.into(), target.into()));

    let hiding = matches!(
        squad.orders.as_slice(),
        [Order::Hide(_), ..] | [Order::MoveFastTo(_) | Order::SneakTo(_), Order::Hide(_)]
    );
    if !hiding
        && squad.under_fire
        && !combat::in_cover(perception, squad)
        && let Some(cover) = combat::withdraw_cover(perception, squad, target)
    {
        return Some(vec![
            Order::MoveFastTo(cover.position),
            hide(cover.position),
        ]);
    }

    match squad.orders.as_slice() {
        [Order::Hide(_), ..] => Some(squad.orders.clone()),
        [Order::MoveFastTo(_) | Order::SneakTo(_), Order::Hide(_)] => Some(squad.orders.clone()),
        [
            move_ @ (Order::MoveFastTo(p) | Order::SneakTo(p)),
            Order::Defend(_) | Order::Engage(_),
            ..,
        ] => Some(vec![move_.clone(), hide(*p)]),
        _ => Some(vec![hide(squad.position)]),
    }
}

/// Offensive AI (see module documentation)
#[derive(Debug, Default)]
pub struct Offensive {
    pace: Pace,
}

impl Commander for Offensive {
    /// Give orders to all squads, during fight phase only
    fn tick(&mut self, perception: &Perception) -> Vec<Command> {
        if !perception.phase.fight() {
            return vec![];
        }

        let squads: Vec<&SquadView> = perception.squads.iter().collect();
        let engagements = combat::engagements(perception, &squads);
        let regroup = self.pace.regroup(perception);

        decide(perception, &squads, &engagements, regroup)
            .into_iter()
            .filter_map(|(squad, orders)| {
                Command::set_orders_if_changed(squad, orders.unwrap_or(vec![Order::Idle]))
            })
            .chain(support::requests(perception))
            .collect()
    }
}

/// Orders for each given squad: combat orders (take cover and engage, help) else capture order
/// (hold order when `regroup`). None when squad is not in combat and no flag is left to capture.
pub(crate) fn decide<'a>(
    perception: &Perception,
    squads: &[&'a SquadView],
    engagements: &[Engagement],
    regroup: bool,
) -> Vec<(&'a SquadView, Option<Vec<Order>>)> {
    // When regrouping, squads in contact disengage, except when the enemy is too close
    let disengage = |squad: &SquadView| {
        regroup
            && perception
                .known_threat(squad.position, combat::CLOSE_COMBAT_DISTANCE)
                .is_none()
    };
    let (fighting, free): (Vec<_>, Vec<_>) = squads
        .iter()
        .map(|squad| match disengage(squad) {
            true => (*squad, None),
            false => (*squad, combat::orders(perception, squad, engagements)),
        })
        .partition(|(_, order)| order.is_some());
    let free: Vec<&SquadView> = free.into_iter().map(|(squad, _)| squad).collect();

    fighting
        .into_iter()
        .chain(capture_orders(perception, free, regroup))
        .collect()
}

/// Capture order for each given squad, spread over flags (None when all flags are controlled).
/// When squads `regroup` (see [`crate::pace`]), they hold their position, defending facing the
/// nearest known enemy.
fn capture_orders<'a>(
    perception: &Perception,
    squads: Vec<&'a SquadView>,
    regroup: bool,
) -> Vec<(&'a SquadView, Option<Vec<Order>>)> {
    if regroup {
        return squads
            .into_iter()
            .map(|squad| {
                crate::instrument(perception, squad, "regroup");
                (squad, hold_orders(perception, squad))
            })
            .collect();
    }

    let targets = capture::targets(perception, &squads);
    // Squads advancing under threat, with their remaining distance to their objective
    let bounding: Vec<(&SquadView, f32)> = squads
        .iter()
        .zip(&targets)
        .filter_map(|(squad, target)| {
            let target = (*target)?;
            let threat = combat::bounding_threat(perception, squad, target)?;
            // Far threat: squads advance from cover to cover without waiting for each other
            (perception.meters(squad.position, threat) <= combat::SUPPRESS_DISTANCE)
                .then_some(())?;
            Some((*squad, perception.meters(squad.position, target)))
        })
        .collect();

    squads
        .into_iter()
        .zip(targets)
        .map(|(squad, target)| {
            let orders = target.map(|target| {
                let threat = combat::bounding_threat(perception, squad, target);
                let remaining = perception.meters(squad.position, target);
                match threat {
                    Some(threat)
                        if perception.meters(squad.position, threat)
                            <= combat::SUPPRESS_DISTANCE
                            && !combat::may_bound(perception, squad, remaining, &bounding) =>
                    {
                        crate::instrument(perception, squad, "overwatch");
                        combat::overwatch_orders(perception, squad, threat)
                    }
                    _ => {
                        crate::instrument(perception, squad, "advance");
                        combat::advance_orders(perception, squad, target, CAPTURE_MOVE_TOLERANCE)
                    }
                }
            });
            (squad, orders)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use oc_individual::{IndividualIndex, squad::SquadIndex};
    use oc_root::{battle::BattlePhase, geo::WorldVec2, side::Side};
    use oc_world::flag::Ownership;

    use super::*;
    use oc_utils::d2::{Angle, Direction};

    use crate::perception::{Candidate, CandidateKind, EnemyView, FlagView};

    fn perception(phase: BattlePhase) -> Perception {
        Perception {
            side: Side::B,
            pixels_per_meter: 1.0,
            phase,
            squads: vec![SquadView {
                index: SquadIndex(3),
                position: WorldVec2::new(0.0, 0.0),
                actives: 4,
                orders: vec![],
                placed: false,
                under_fire: false,
                smokes: 0,
            }],
            enemies: vec![],
            flags: vec![
                FlagView {
                    center: WorldVec2::new(1000.0, 0.0),
                    ownership: Ownership::A,
                    enemies: vec![],
                },
                FlagView {
                    center: WorldVec2::new(10.0, 0.0),
                    ownership: Ownership::B,
                    enemies: vec![],
                },
            ],
            supports: vec![],
            progress: 0.0,
            morale: 1.0,
            enemy_spawns: vec![],
            candidates: vec![],
            covers: vec![],
        }
    }

    #[test]
    fn nothing_during_deployment() {
        assert!(
            Offensive::default()
                .tick(&perception(BattlePhase::Deployment))
                .is_empty()
        );
    }

    #[test]
    fn go_to_nearest_not_owned_flag() {
        let commands = Offensive::default().tick(&perception(BattlePhase::Fight));
        assert_eq!(
            commands,
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::MoveTo(WorldVec2::new(1000.0, 0.0))]
            )]
        );
    }

    #[test]
    fn spread_squads_over_flags() {
        let mut perception = perception(BattlePhase::Fight);
        perception.flags[1].ownership = Ownership::A;
        perception.flags[1].center = WorldVec2::new(1100.0, 0.0);
        let mut second = perception.squads[0].clone();
        second.index = SquadIndex(4);
        perception.squads.push(second);
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![
                Command::SetOrders(
                    SquadIndex(3),
                    vec![Order::MoveTo(WorldVec2::new(1000.0, 0.0))]
                ),
                Command::SetOrders(
                    SquadIndex(4),
                    vec![Order::MoveTo(WorldVec2::new(1100.0, 0.0))]
                ),
            ]
        );
    }

    #[test]
    fn go_to_contested_flag() {
        let mut perception = perception(BattlePhase::Fight);
        perception.flags[0].ownership = Ownership::Nobody;
        perception.flags[1].ownership = Ownership::Both;
        let commands = Offensive::default().tick(&perception);
        assert_eq!(
            commands,
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::MoveTo(WorldVec2::new(10.0, 0.0))]
            )]
        );
    }

    #[test]
    fn engage_near_enemy() {
        let mut perception = perception(BattlePhase::Fight);
        perception.enemies = vec![
            EnemyView {
                index: IndividualIndex(7),
                position: WorldVec2::new(100.0, 0.0),
                seen_by: vec![SquadIndex(3)],
            },
            EnemyView {
                index: IndividualIndex(8),
                position: WorldVec2::new(500.0, 0.0),
                seen_by: vec![SquadIndex(3)],
            },
        ];
        let commands = Offensive::default().tick(&perception);
        assert_eq!(
            commands,
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::Engage(IndividualIndex(7))]
            )]
        );
    }

    /// Cover of squad 3
    fn cover(x: f32, y: f32, kind: CandidateKind, cover: f32) -> (SquadIndex, Candidate) {
        (
            SquadIndex(3),
            Candidate {
                position: WorldVec2::new(x, y),
                kind,
                angle: Angle::zero(),
                cover,
            },
        )
    }

    /// Squad 3 at (0, 0) sees enemy 7 at (100, 0)
    fn contact() -> Perception {
        let mut perception = perception(BattlePhase::Fight);
        perception.enemies = vec![EnemyView {
            index: IndividualIndex(7),
            position: WorldVec2::new(100.0, 0.0),
            seen_by: vec![SquadIndex(3)],
        }];
        perception
    }

    /// Defend order from `from` facing `enemy`
    fn defend(from: (f32, f32), enemy: (f32, f32)) -> Order {
        Order::Defend(Direction::from_points2d(from.into(), enemy.into()))
    }

    #[test]
    fn take_cover_and_defend_facing_enemy() {
        let mut perception = contact();
        // Enemy is not near: squad runs to its cover
        perception.enemies[0].position = WorldVec2::new(120.0, 0.0);
        perception.covers = vec![
            // Too far from squad
            cover(-40.0, 0.0, CandidateKind::Interior, 1.0),
            // Too near enemy
            cover(105.0, 0.0, CandidateKind::Interior, 1.0),
            // Best one: interior, a little toward enemy
            cover(15.0, 10.0, CandidateKind::Interior, 0.5),
            // Open, less opaque
            cover(-10.0, 5.0, CandidateKind::Open, 0.6),
        ];
        let defend = defend((15.0, 10.0), (120.0, 0.0));
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![
                    Order::MoveFastTo(WorldVec2::new(15.0, 10.0)),
                    defend.clone()
                ]
            )]
        );

        // Running to cover: orders are kept, even if a better cover appears
        perception.squads[0].orders = vec![
            Order::MoveFastTo(WorldVec2::new(15.0, 10.0)),
            defend.clone(),
        ];
        perception.squads[0].position = WorldVec2::new(8.0, 5.0);
        perception
            .covers
            .push(cover(8.0, 15.0, CandidateKind::Interior, 1.0));
        assert!(Offensive::default().tick(&perception).is_empty());

        // Cover reached (move accomplished): defend from it, even if enemy moves a little
        perception.squads[0].orders = vec![defend];
        perception.squads[0].position = WorldVec2::new(15.0, 10.0);
        perception.enemies[0].position = WorldVec2::new(100.0, 20.0);
        assert!(Offensive::default().tick(&perception).is_empty());
    }

    #[test]
    fn defend_from_position_when_in_cover() {
        let mut perception = contact();
        perception.covers = vec![
            cover(3.0, 0.0, CandidateKind::Open, 0.5),
            cover(15.0, 10.0, CandidateKind::Interior, 1.0),
        ];
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![defend((0.0, 0.0), (100.0, 0.0))]
            )]
        );
    }

    #[test]
    fn no_cover_change_while_engaging() {
        let mut perception = contact();
        perception.squads[0].orders = vec![Order::Engage(IndividualIndex(7))];
        perception.covers = vec![cover(15.0, 10.0, CandidateKind::Interior, 1.0)];
        assert!(Offensive::default().tick(&perception).is_empty());
    }

    #[test]
    fn ignore_covers_of_other_squads() {
        let mut perception = contact();
        let (_, other) = cover(15.0, 10.0, CandidateKind::Interior, 1.0);
        perception.covers = vec![(SquadIndex(4), other)];
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::Engage(IndividualIndex(7))]
            )]
        );
    }

    /// Squad 3 at (0, 0) is shot at by an unseen enemy inside the flag at (10, 0) (enemy at
    /// (100, 0))
    fn under_fire() -> Perception {
        let mut perception = perception(BattlePhase::Fight);
        perception.squads[0].under_fire = true;
        perception.flags[1].enemies = vec![WorldVec2::new(100.0, 0.0)];
        perception
    }

    #[test]
    fn under_fire_take_cover_and_defend() {
        let mut perception = under_fire();
        perception.covers = vec![cover(-10.0, 5.0, CandidateKind::Open, 0.6)];
        let defend = defend((-10.0, 5.0), (100.0, 0.0));
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![
                    Order::MoveFastTo(WorldVec2::new(-10.0, 5.0)),
                    defend.clone()
                ]
            )]
        );

        // Defending: kept while under fire
        perception.squads[0].orders = vec![defend];
        assert!(Offensive::default().tick(&perception).is_empty());

        // Fire stopped: advance again, crawling (enemy known in flag, no cover toward objective)
        perception.squads[0].under_fire = false;
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::SneakTo(WorldVec2::new(15.0, 0.0))]
            )]
        );
    }

    #[test]
    fn under_fire_without_cover_defend_in_place() {
        let perception = under_fire();
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![defend((0.0, 0.0), (100.0, 0.0))]
            )]
        );
    }

    #[test]
    fn engage_from_position_when_no_cover() {
        let mut perception = contact();
        perception.covers = vec![cover(90.0, 0.0, CandidateKind::Interior, 1.0)];
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::Engage(IndividualIndex(7))]
            )]
        );
    }

    /// Squad 3 at (0, 0) attacks flag at (1000, 0), enemy 7 at (100, 20) is seen by another
    /// squad
    fn enemy_known() -> Perception {
        let mut perception = perception(BattlePhase::Fight);
        perception.enemies = vec![EnemyView {
            index: IndividualIndex(7),
            position: WorldVec2::new(100.0, 20.0),
            seen_by: vec![SquadIndex(42)],
        }];
        perception
    }

    #[test]
    fn advance_from_cover_to_cover_when_enemy_known() {
        let mut perception = enemy_known();
        perception.covers = vec![
            // Behind squad
            cover(-10.0, 0.0, CandidateKind::Interior, 1.0),
            // Not enough progress
            cover(3.0, 5.0, CandidateKind::Interior, 1.0),
            // Too far from squad
            cover(40.0, 0.0, CandidateKind::Interior, 1.0),
            // Less progress
            cover(10.0, 5.0, CandidateKind::Open, 0.5),
            // Best one
            cover(25.0, 0.0, CandidateKind::Open, 0.5),
        ];
        let defend = defend((25.0, 0.0), (100.0, 20.0));
        // Cover is near the enemy: squad crawls to it
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::SneakTo(WorldVec2::new(25.0, 0.0)), defend.clone()]
            )]
        );

        // Crawling to cover: orders are kept
        perception.squads[0].orders = vec![Order::SneakTo(WorldVec2::new(25.0, 0.0)), defend];
        perception.squads[0].position = WorldVec2::new(10.0, 0.0);
        assert!(Offensive::default().tick(&perception).is_empty());
    }

    #[test]
    fn crawl_when_enemy_known_and_no_cover() {
        let mut perception = enemy_known();
        let crawl = Order::SneakTo(WorldVec2::new(15.0, 0.0));
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(SquadIndex(3), vec![crawl.clone()])]
        );

        // Crawling: order is kept
        perception.squads[0].orders = vec![crawl];
        perception.squads[0].position = WorldVec2::new(5.0, 0.0);
        assert!(Offensive::default().tick(&perception).is_empty());
    }

    #[test]
    fn advance_from_cover_to_cover_when_enemy_remembered() {
        let perception = perception(BattlePhase::Fight).with_remembered(vec![EnemyView {
            index: IndividualIndex(7),
            position: WorldVec2::new(100.0, 20.0),
            seen_by: vec![],
        }]);
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::SneakTo(WorldVec2::new(15.0, 0.0))]
            )]
        );
    }

    #[test]
    fn move_when_known_enemy_is_far() {
        let mut perception = enemy_known();
        perception.enemies[0].position = WorldVec2::new(250.0, 0.0);
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::MoveTo(WorldVec2::new(1000.0, 0.0))]
            )]
        );
    }

    #[test]
    fn contested_flag_goes_to_enemy_inside() {
        let mut perception = perception(BattlePhase::Fight);
        perception.flags[1].ownership = Ownership::Both;
        perception.flags[1].enemies = vec![WorldVec2::new(12.0, 5.0), WorldVec2::new(30.0, 0.0)];
        // Enemies are known inside the flag: crawl toward the nearest one (no cover found)
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::SneakTo(WorldVec2::new(12.0, 5.0))]
            )]
        );
    }

    #[test]
    fn help_near_squad_in_combat() {
        let mut perception = perception(BattlePhase::Fight);
        let mut fighting = perception.squads[0].clone();
        fighting.index = SquadIndex(4);
        fighting.position = WorldVec2::new(50.0, 0.0);
        perception.squads.push(fighting);
        perception.enemies = vec![EnemyView {
            index: IndividualIndex(7),
            position: WorldVec2::new(150.0, 0.0),
            seen_by: vec![SquadIndex(4)],
        }];

        // Helping squad advances toward the enemy engaged by helped squad (crawls: enemy is
        // near and no cover is found)
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![
                Command::SetOrders(
                    SquadIndex(3),
                    vec![Order::SneakTo(WorldVec2::new(15.0, 0.0))]
                ),
                Command::SetOrders(SquadIndex(4), vec![Order::Engage(IndividualIndex(7))]),
            ]
        );

        // Enemy moved a little: current crawl is kept
        perception.squads[0].orders = vec![Order::SneakTo(WorldVec2::new(15.0, 0.0))];
        perception.squads[1].orders = vec![Order::Engage(IndividualIndex(7))];
        perception.enemies[0].position = WorldVec2::new(145.0, 0.0);
        assert!(Offensive::default().tick(&perception).is_empty());

        // Helping squad approached and sees the enemy: it engages
        perception.squads[0].position = WorldVec2::new(100.0, 0.0);
        perception.enemies[0].seen_by.push(SquadIndex(3));
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::Engage(IndividualIndex(7))]
            )]
        );
    }

    #[test]
    fn hiding_squad_in_combat_defends_again() {
        // Squad hid during a regroup, the attack resumed: it fires back from its position
        let mut perception = contact();
        let direction = Direction::from_points2d((0.0, 0.0).into(), (100.0, 0.0).into());
        perception.squads[0].orders = vec![Order::Hide(direction)];
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::Defend(direction)]
            )]
        );
    }

    #[test]
    fn no_command_when_order_unchanged() {
        let mut perception = perception(BattlePhase::Fight);
        perception.squads[0].orders = vec![Order::MoveTo(WorldVec2::new(1000.0, 0.0))];
        assert!(Offensive::default().tick(&perception).is_empty());
    }

    #[test]
    fn crawl_to_near_cover_when_enemy_near() {
        let mut perception = contact();
        perception.covers = vec![
            // Too far to crawl to it while enemy is near
            cover(15.0, 10.0, CandidateKind::Interior, 1.0),
            cover(8.0, 5.0, CandidateKind::Open, 0.6),
        ];
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![
                    Order::SneakTo(WorldVec2::new(8.0, 5.0)),
                    defend((8.0, 5.0), (100.0, 0.0))
                ]
            )]
        );
    }

    #[test]
    fn under_fire_by_far_seen_enemy_take_cover() {
        let mut perception = perception(BattlePhase::Fight);
        perception.squads[0].under_fire = true;
        // Seen, but beyond engage distance
        perception.enemies = vec![EnemyView {
            index: IndividualIndex(7),
            position: WorldVec2::new(180.0, 0.0),
            seen_by: vec![SquadIndex(3)],
        }];
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![defend((0.0, 0.0), (180.0, 0.0))]
            )]
        );
    }

    /// Squads 3 (at (0, 0)) and 4 (at (0, 20)) attack flag at (1000, 0), enemy 7 at (100, 0) is
    /// seen by another squad
    fn two_squads_under_threat() -> Perception {
        let mut perception = enemy_known();
        perception.enemies[0].position = WorldVec2::new(100.0, 0.0);
        let mut second = perception.squads[0].clone();
        second.index = SquadIndex(4);
        second.position = WorldVec2::new(0.0, 20.0);
        perception.squads.push(second);
        perception
    }

    #[test]
    fn bounding_overwatch() {
        let mut perception = two_squads_under_threat();
        // Squad 4 is ahead: rear-most squad 3 moves (crawls, no cover) while 4 suppresses
        perception.squads[1].position = WorldVec2::new(10.0, 20.0);
        let crawl = Order::SneakTo(WorldVec2::new(15.0, 0.0));
        let suppress = Order::Suppress(WorldVec2::new(100.0, 0.0));
        assert_eq!(
            Offensive::default().tick(&perception),
            vec![
                Command::SetOrders(SquadIndex(3), vec![crawl.clone()]),
                Command::SetOrders(SquadIndex(4), vec![suppress.clone()]),
            ]
        );

        // Squad 3 moved ahead and is now defending: squad 4 (rear-most) moves, 3 suppresses
        perception.squads[0].orders = vec![defend((20.0, 0.0), (100.0, 0.0))];
        perception.squads[0].position = WorldVec2::new(20.0, 0.0);
        perception.squads[1].orders = vec![suppress];
        let commands = Offensive::default().tick(&perception);
        assert!(commands.contains(&Command::SetOrders(
            SquadIndex(3),
            vec![Order::Suppress(WorldVec2::new(100.0, 0.0))]
        )));
        assert!(commands.iter().any(|c| matches!(
            c,
            Command::SetOrders(SquadIndex(4), orders) if matches!(orders.first(), Some(Order::SneakTo(_)))
        )));
    }

    #[test]
    fn regroup_after_costly_wave() {
        let mut offensive = Offensive::default();
        let mut perception = enemy_known();
        assert_eq!(
            offensive.tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::SneakTo(WorldVec2::new(15.0, 0.0))]
            )]
        );

        // Wave lost too much: squad hides at its position
        perception.morale = 1.0 - crate::pace::WAVE_LOSSES - 0.01;
        perception.progress = 0.2;
        assert_eq!(
            offensive.tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::Hide(Direction::from_points2d(
                    (0.0, 0.0).into(),
                    (100.0, 20.0).into()
                ))]
            )]
        );

        // Regroup ended: attack again
        perception.progress = 0.2 + crate::pace::regroup_duration(1.0 - perception.morale);
        assert_eq!(
            offensive.tick(&perception),
            vec![Command::SetOrders(
                SquadIndex(3),
                vec![Order::SneakTo(WorldVec2::new(15.0, 0.0))]
            )]
        );
    }
}
