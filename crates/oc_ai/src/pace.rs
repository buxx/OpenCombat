//! Attack pace: attacks come in waves. When a wave costs the side [`WAVE_LOSSES`] of its
//! individuals, squads stop advancing and regroup (hide) during a part of the battle duration
//! (see [`regroup_duration`]), then a new wave starts. Regroups get longer as the side losses
//! grow: attacks run out of steam instead of continuing until the side breaks, and the battle
//! never freezes (attack always resumes).
use crate::perception::Perception;

/// Part of side individuals a wave can lose before squads regroup
pub const WAVE_LOSSES: f32 = 0.08;
/// Regroup duration (as part of battle duration) of a side without losses (54 seconds of a 15
/// minutes battle): a short pause to take cover, not a long wait
pub const REGROUP_PROGRESS: f32 = 0.06;
/// Side losses which double regroup duration (see [`regroup_duration`])
pub const REGROUP_LOSSES_SCALE: f32 = 0.3;
/// Maximal regroup duration (as part of battle duration, 72 seconds of a 15 minutes battle):
/// squads never hide too long
pub const REGROUP_MAX_PROGRESS: f32 = 0.08;

/// Regroup duration (as part of battle duration) of a side having lost `losses` (part of its
/// individuals): [`REGROUP_PROGRESS`], plus the same for each [`REGROUP_LOSSES_SCALE`] lost, up
/// to [`REGROUP_MAX_PROGRESS`]
pub fn regroup_duration(losses: f32) -> f32 {
    (REGROUP_PROGRESS * (1.0 + losses / REGROUP_LOSSES_SCALE)).min(REGROUP_MAX_PROGRESS)
}

/// Pace of the attack of one side
#[derive(Debug, Clone, Default)]
pub struct Pace {
    /// Side losses (part of its individuals) at current wave start
    wave_start: f32,
    /// Battle progress when regroup ends (None when attacking)
    regroup_until: Option<f32>,
}

impl Pace {
    /// Update pace from `perception`: true if squads must regroup (not attack)
    pub fn regroup(&mut self, perception: &Perception) -> bool {
        let losses = 1.0 - perception.morale;

        if let Some(until) = self.regroup_until {
            if perception.progress < until {
                return true;
            }
            #[cfg(feature = "battle_instrument")]
            tracing::info!(name="battle-pace-attack", side=?perception.side, losses=losses, progress=perception.progress);
            self.regroup_until = None;
            self.wave_start = losses;
        }

        if losses - self.wave_start >= WAVE_LOSSES {
            #[cfg(feature = "battle_instrument")]
            tracing::info!(name="battle-pace-regroup", side=?perception.side, losses=losses, progress=perception.progress);
            self.regroup_until = Some(perception.progress + regroup_duration(losses));
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use oc_root::{battle::BattlePhase, side::Side};

    use super::*;

    fn perception(morale: f32, progress: f32) -> Perception {
        Perception {
            side: Side::A,
            pixels_per_meter: 1.0,
            phase: BattlePhase::Fight,
            squads: vec![],
            enemies: vec![],
            flags: vec![],
            supports: vec![],
            progress,
            morale,
            enemy_spawns: vec![],
            candidates: vec![],
            covers: vec![],
        }
    }

    #[test]
    fn waves_and_regroups() {
        let wave = WAVE_LOSSES + 0.01;
        let duration = regroup_duration(wave);
        let mut pace = Pace::default();
        assert!(!pace.regroup(&perception(1.0, 0.0)));
        assert!(!pace.regroup(&perception(1.0 - wave / 2.0, 0.1)));
        // Wave lost too much: regroup
        assert!(pace.regroup(&perception(1.0 - wave, 0.2)));
        assert!(pace.regroup(&perception(1.0 - wave - 0.05, 0.2 + duration / 2.0)));
        // Regroup ended: new wave, from current losses
        let losses = wave + 0.05;
        let end = 0.2 + duration;
        assert!(!pace.regroup(&perception(1.0 - losses, end)));
        assert!(!pace.regroup(&perception(1.0 - losses - wave / 2.0, end + 0.01)));
        assert!(pace.regroup(&perception(1.0 - losses - wave, end + 0.02)));
    }

    #[test]
    fn regroups_get_longer_with_losses() {
        assert_eq!(regroup_duration(0.0), REGROUP_PROGRESS);
        assert!(regroup_duration(0.05) > REGROUP_PROGRESS);
        assert!(regroup_duration(0.1) > regroup_duration(0.05));
        assert_eq!(regroup_duration(0.9), REGROUP_MAX_PROGRESS);
    }
}
