//! Support (artillery, etc.) requests shared by AIs. Only enemies currently seen by the side are
//! targeted (no memory, no cheat). An available support is kept for the second half of the
//! battle (or when the side morale fell under 75%): then it is requested on the densest group of
//! seen enemies (a single seen enemy is enough). Before that, it is only requested on a crowd of
//! more than [`CROWD_MIN_ENEMIES`] seen enemies (within [`CROWD_RADIUS`] meters). No support is
//! requested when a squad of the side is near the area where shells fall.
use oc_root::geo::WorldVec2;

use crate::{
    command::Command,
    perception::{Perception, SupportView, centroid},
};

/// Minimal count of seen enemies in the target area to request a support (late battle or low
/// morale). One seen enemy is enough: its squad is around it.
pub const SUPPORT_MIN_ENEMIES: usize = 1;
/// Part of the support radius (around a seen enemy) where enemies are counted as targeted (shells
/// are denser near the requested position)
pub const SUPPORT_TARGET_FACTOR: f32 = 0.6;
/// Distance (meters), added to the support radius, under which a squad of the side forbids a
/// request (its members are spread around its position)
pub const SUPPORT_FRIENDLY_MARGIN: f32 = 10.0;
/// Battle progress (see [`Perception::progress`]) from which supports are used
pub const SUPPORT_FROM_PROGRESS: f32 = 0.5;
/// Side morale (see [`Perception::morale`]) under which supports are used
pub const SUPPORT_UNDER_MORALE: f32 = 0.75;
/// Seen enemies count, within [`CROWD_RADIUS`], above which a support is used at any time
pub const CROWD_MIN_ENEMIES: usize = 10;
/// Radius (meters) of a crowd area (50 meters wide)
pub const CROWD_RADIUS: f32 = 25.0;

/// Support requests to send now: one per available support type having a target (see
/// [`target`])
pub fn requests(perception: &Perception) -> Vec<Command> {
    if !perception.phase.fight() {
        return vec![];
    }

    perception
        .supports
        .iter()
        .filter(|s| s.available > 0)
        .filter_map(|support| {
            let position = target(perception, support)?;
            tracing::debug!(name="ai-support-request", side=?perception.side, type_=%support.type_, position=?position, progress=perception.progress, morale=perception.morale);
            Some(Command::RequestSupport(support.type_.clone(), position))
        })
        .collect()
}

/// True if supports are used on any group of enemies: second half of the battle, or side morale
/// under [`SUPPORT_UNDER_MORALE`]
fn time_to_use(perception: &Perception) -> bool {
    perception.progress >= SUPPORT_FROM_PROGRESS || perception.morale < SUPPORT_UNDER_MORALE
}

/// Center of the targeted group of seen enemies: the densest crowd (see [`CROWD_MIN_ENEMIES`]),
/// else (only when [`time_to_use`]) the densest group of at least [`SUPPORT_MIN_ENEMIES`] within
/// [`SUPPORT_TARGET_FACTOR`] of support radius. Groups near squads of the side are ignored. Between
/// groups of the same size, the nearest to squads of the side (most threatening) is chosen.
fn target(perception: &Perception, support: &SupportView) -> Option<WorldVec2> {
    let crowd = densest(perception, support, CROWD_RADIUS, CROWD_MIN_ENEMIES + 1);
    let group = || {
        let reach = support.radius * SUPPORT_TARGET_FACTOR;
        densest(perception, support, reach, SUPPORT_MIN_ENEMIES)
    };

    crowd.or_else(|| time_to_use(perception).then(group).flatten())
}

/// Center of the densest group of seen enemies (at least `min` within `reach` meters of one of
/// them), far enough from squads of the side to be targeted by `support`
fn densest(
    perception: &Perception,
    support: &SupportView,
    reach: f32,
    min: usize,
) -> Option<WorldVec2> {
    let enemies: Vec<WorldVec2> = perception
        .enemies
        .iter()
        .filter(|e| !e.seen_by.is_empty())
        .map(|e| e.position)
        .collect();
    let safe = support.radius + SUPPORT_FRIENDLY_MARGIN;

    enemies
        .iter()
        .map(|enemy| {
            let group: Vec<WorldVec2> = enemies
                .iter()
                .filter(|other| perception.meters(*enemy, **other) <= reach)
                .copied()
                .collect();
            group
        })
        .filter(|group| group.len() >= min)
        .filter_map(|group| Some((group.len(), centroid(&group)?)))
        .filter(|(_, center)| nearest_squad(perception, *center) > safe)
        .max_by(|(a, a_center), (b, b_center)| {
            let threat = |center: &WorldVec2| -nearest_squad(perception, *center);
            a.cmp(b).then(threat(a_center).total_cmp(&threat(b_center)))
        })
        .map(|(_, center)| center)
}

/// Distance (meters) from `position` to the nearest squad of the side
fn nearest_squad(perception: &Perception, position: WorldVec2) -> f32 {
    perception
        .squads
        .iter()
        .map(|squad| perception.meters(squad.position, position))
        .fold(f32::MAX, f32::min)
}

#[cfg(test)]
mod tests {
    use oc_individual::{IndividualIndex, squad::SquadIndex};
    use oc_root::{battle::BattlePhase, side::Side};
    use oc_world::support::SupportType;

    use super::*;
    use crate::perception::{EnemyView, SquadView};

    fn artillery(available: u8) -> SupportView {
        SupportView {
            type_: SupportType::new("Artillery"),
            available,
            radius: 45.0,
        }
    }

    fn enemy(i: u64, x: f32, y: f32) -> EnemyView {
        EnemyView {
            index: IndividualIndex(i),
            position: WorldVec2::new(x, y),
            seen_by: vec![SquadIndex(0)],
        }
    }

    fn request(x: f32, y: f32) -> Vec<Command> {
        vec![Command::RequestSupport(
            SupportType::new("Artillery"),
            WorldVec2::new(x, y),
        )]
    }

    /// Second half of the battle. Squad at (0, 0), seen enemies: a group of 3 around (200, 0), a
    /// lone one at (100, 100)
    fn perception() -> Perception {
        Perception {
            side: Side::A,
            pixels_per_meter: 1.0,
            phase: BattlePhase::Fight,
            squads: vec![SquadView {
                index: SquadIndex(0),
                position: WorldVec2::new(0.0, 0.0),
                actives: 4,
                orders: vec![],
                placed: false,
                under_fire: false,
                smokes: 0,
            }],
            enemies: vec![
                enemy(1, 195.0, 0.0),
                enemy(2, 205.0, 0.0),
                enemy(3, 200.0, 15.0),
                enemy(4, 100.0, 100.0),
            ],
            flags: vec![],
            supports: vec![artillery(1)],
            progress: 0.6,
            morale: 1.0,
            enemy_spawns: vec![],
            candidates: vec![],
            covers: vec![],
        }
    }

    /// `count` seen enemies spread over a line of 40 meters around (200, 0)
    fn crowd(count: u64) -> Vec<EnemyView> {
        let step = 40.0 / (count - 1) as f32;
        (0..count)
            .map(|i| enemy(i, 180.0 + i as f32 * step, 0.0))
            .collect()
    }

    #[test]
    fn request_on_enemy_group_in_second_half() {
        assert_eq!(requests(&perception()), request(200.0, 5.0));
    }

    #[test]
    fn keep_support_during_first_half() {
        let mut perception = perception();
        perception.progress = 0.4;
        assert!(requests(&perception).is_empty());
    }

    #[test]
    fn request_when_morale_under_threshold() {
        let mut perception = perception();
        perception.progress = 0.1;
        perception.morale = 0.7;
        assert_eq!(requests(&perception), request(200.0, 5.0));

        perception.morale = 0.8;
        assert!(requests(&perception).is_empty());
    }

    #[test]
    fn request_on_crowd_at_any_time() {
        let mut perception = perception();
        perception.progress = 0.0;
        perception.enemies = crowd(11);
        assert_eq!(requests(&perception), request(200.0, 0.0));

        // 10 enemies is not a crowd
        perception.enemies = crowd(10);
        assert!(requests(&perception).is_empty());
    }

    #[test]
    fn only_target_seen_enemies() {
        let mut perception = perception();
        // Remembered (not seen anymore) enemies
        for enemy in &mut perception.enemies {
            enemy.seen_by.clear();
        }
        perception.flags = vec![crate::perception::FlagView {
            center: WorldVec2::new(200.0, 0.0),
            ownership: oc_world::flag::Ownership::B,
            // Known inside flag only (AI cheat)
            enemies: vec![WorldVec2::new(200.0, 15.0), WorldVec2::new(200.0, -15.0)],
        }];
        assert!(requests(&perception).is_empty());
    }

    #[test]
    fn no_request_when_no_support_available() {
        let mut perception = perception();
        perception.supports = vec![artillery(0)];
        assert!(requests(&perception).is_empty());
    }

    #[test]
    fn request_on_lone_enemy() {
        let mut perception = perception();
        perception.enemies = vec![enemy(4, 100.0, 100.0)];
        assert_eq!(requests(&perception), request(100.0, 100.0));
    }

    #[test]
    fn no_request_near_friendly_squad() {
        let mut perception = perception();
        perception.enemies.truncate(3);
        perception.squads[0].position = WorldVec2::new(150.0, 0.0);
        assert!(requests(&perception).is_empty());
    }

    #[test]
    fn prefer_group_nearest_to_squads() {
        let mut perception = perception();
        perception.enemies = vec![
            enemy(1, 395.0, 0.0),
            enemy(2, 405.0, 0.0),
            enemy(3, 400.0, 15.0),
            enemy(4, 0.0, 195.0),
            enemy(5, 0.0, 205.0),
            enemy(6, 15.0, 200.0),
        ];
        assert_eq!(requests(&perception), request(5.0, 200.0));
    }

    #[test]
    fn nothing_out_of_fight() {
        let mut perception = perception();
        perception.phase = BattlePhase::Deployment;
        assert!(requests(&perception).is_empty());
    }
}
