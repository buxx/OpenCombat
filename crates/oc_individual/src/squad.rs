use derive_more::Constructor;
use getset::WithSetters;
use glam::Vec2;
use oc_mod::illustration::IllustrationIndex;
use oc_root::{WorldConfig, geo::WorldVec2, side::Side, y::V};
use oc_utils::d2::Angle;

use crate::{
    IndividualIndex,
    order::{Order, OrderIndex},
};

#[oc_derive::archivable(serde)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Constructor, Hash)]
pub struct SquadIndex(pub u64);

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, WithSetters)]
pub struct Squad {
    /// Side of the squad
    #[getset(set_with = "pub")]
    pub side: Side,
    /// Illustration
    #[getset(set_with = "pub")]
    pub illustration: IllustrationIndex,
    /// Individual identifiers in this squad. Integrity must be checked before game run
    #[getset(set_with = "pub")]
    pub members: Vec<IndividualIndex>,
    /// Current leader (must be a member and, when possible, can_leadership)
    #[getset(set_with = "pub")]
    pub leader: IndividualIndex,
    /// Number of alive members
    #[getset(set_with = "pub")]
    pub actives: u8,
    /// Formation of squad (to place members)
    #[getset(set_with = "pub")]
    pub formation: SquadFormation,
    /// Order given to this squad.
    #[getset(set_with = "pub")]
    pub orders: Vec<Order>,
    /// Computed position of the squad (leader position)
    #[getset(set_with = "pub")]
    pub position: WorldVec2,
    /// Computed status
    #[getset(set_with = "pub")]
    pub status: SquadStatus,
}

impl Squad {
    pub fn fresh(side: Side, members: Vec<IndividualIndex>, position: WorldVec2) -> Self {
        let actives = members.len() as u8;
        let leader = *members
            .first()
            .expect("We delegate insurance there is member at the start of program");
        Self {
            leader,
            side,
            illustration: IllustrationIndex(0),
            members,
            actives,
            formation: SquadFormation::Line,
            orders: vec![],
            position,
            status: SquadStatus::Operational,
        }
    }

    pub fn leader(&self) -> IndividualIndex {
        self.leader
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SquadStatus {
    Operational,
    Casualities,
    PartiallyDestroyed,
    Destroyed,
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum Update {
    SetOrders(Vec<Order>),
    SetOrderPosition(OrderIndex, WorldVec2),
    SetPosition(WorldVec2),
    SetActives(u8),
    SetLeader(IndividualIndex),
    SetStatus(SquadStatus),
    Accomplished,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SquadFormation {
    /// Formation make a line, squad leader at center
    Line,
}

impl SquadFormation {
    pub fn positions(
        &self,
        w: &WorldConfig,
        v: V,
        reference: Vec2,
        angle: Angle,
        count: usize,
    ) -> Vec<Vec2> {
        let mut positions = Vec::with_capacity(count);
        positions.push(reference);

        match self {
            SquadFormation::Line => {
                if count == 0 {
                    return vec![reference];
                }

                let dir_x = f32::cos(angle.0);
                let dir_y = match v {
                    V::Server => f32::sin(angle.0),
                    V::Gui => -f32::sin(angle.0),
                };
                let space =
                    w.formation_tiles_between_positions() as f32 * w.geo_pixels_per_tile() as f32;

                // Leader is already at `reference`. Remaining members fan out
                // alternately left/right, one rank further out each pair:
                // -1, +1, -2, +2, -3, +3, ...
                for i in 1..count {
                    let rank = i.div_ceil(2) as f32; // 1,1,2,2,3,3,...
                    let side = if i % 2 == 1 { -1.0 } else { 1.0 };
                    let offset = side * rank * space;

                    let x = reference.x + dir_x * offset;
                    let y = reference.y + dir_y * offset;
                    positions.push(Vec2::new(x, y));
                }
            }
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oc_root::y::V;
    use oc_utils::d2::Direction;
    use rstest::rstest;

    #[rstest]
    //
    #[case(1, Angle::zero(), vec![Vec2::new(100., 100.)])]
    #[case(2, Angle::zero(), vec![Vec2::new(100., 100.), Vec2::new(80., 100.)])]
    #[case(3, Angle::zero(), vec![Vec2::new(100., 100.), Vec2::new(80., 100.), Vec2::new(120., 100.)])]
    #[case(4, Angle::zero(), vec![Vec2::new(100., 100.), Vec2::new(80., 100.), Vec2::new(120., 100.), Vec2::new(60., 100.)])]
    #[case(5, Angle::zero(), vec![Vec2::new(100., 100.), Vec2::new(80., 100.), Vec2::new(120., 100.), Vec2::new(60., 100.), Vec2::new(140., 100.)])]
    //
    #[case(1, Direction::EST.angle(V::Server), vec![Vec2::new(100., 100.)])]
    #[case(2, Direction::EST.angle(V::Server), vec![Vec2::new(100., 100.), Vec2::new(100., 80.)])]
    #[case(3, Direction::EST.angle(V::Server), vec![Vec2::new(100., 100.), Vec2::new(100., 80.), Vec2::new(100., 120.)])]
    fn test_formation_line_positions(
        #[case] count: usize,
        #[case] angle: Angle,
        #[case] expected: Vec<Vec2>,
    ) {
        // Given
        let w = WorldConfig::new(100, 100).with_geo_pixels_per_tile(10);
        let reference = Vec2::new(100., 100.);
        let formation = SquadFormation::Line;

        // When
        let positions = formation.positions(&w, V::Gui, reference, angle, count);

        // Then
        assert_eq!(positions, expected);
    }
}
