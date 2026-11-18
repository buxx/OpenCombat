use crate::side::Side;

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BattlePhase {
    Deployment,
    Fight,
    End(End),
}

impl BattlePhase {
    pub fn fight(&self) -> bool {
        match self {
            BattlePhase::Deployment => false,
            BattlePhase::Fight => true,
            BattlePhase::End(_) => false,
        }
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum End {
    Time(Result),
    Morale(Side, Result),
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Result {
    pub a: SideResult,
    pub b: SideResult,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SideResult {
    TotalVictory,
    Victory,
    Mitigate,
    Defeat,
    TotalDefeat,
}

impl Result {
    /// Determine each side result from the morale (percent) of both sides
    pub fn from_morale(a: f32, b: f32, broken_below: f32) -> Self {
        let (a_result, b_result) = if (a - b).abs() < f32::EPSILON {
            (SideResult::Mitigate, SideResult::Mitigate)
        } else if a > b {
            (
                SideResult::winner(b < broken_below),
                SideResult::loser(b < broken_below),
            )
        } else {
            (
                SideResult::loser(a < broken_below),
                SideResult::winner(a < broken_below),
            )
        };

        Self {
            a: a_result,
            b: b_result,
        }
    }
}

impl SideResult {
    fn winner(total: bool) -> Self {
        match total {
            true => SideResult::TotalVictory,
            false => SideResult::Victory,
        }
    }

    fn loser(total: bool) -> Self {
        match total {
            true => SideResult::TotalDefeat,
            false => SideResult::Defeat,
        }
    }
}
