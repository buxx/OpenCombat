use std::collections::HashMap;

use getset::WithSetters;
use oc_root::uuids::SquadUuid;
use oc_root::{geo::WorldVec2, side::BySide};
use oc_utils::{d2::Direction, polygon::Polygon};
use oc_world::spawn::SpawnZone;

#[derive(Debug, WithSetters)]
pub struct Placer {
    places: Vec<WorldVec2>,
    assigned: HashMap<SquadUuid, WorldVec2>,
    #[getset(set_with = "pub")]
    orders: BySide<OrderPolicy>,
}

impl Placer {
    pub fn new(places: Vec<WorldVec2>) -> Self {
        Self {
            places,
            assigned: HashMap::default(),
            orders: BySide::default(),
        }
    }

    pub fn with_assigned(mut self, places: Vec<(SquadUuid, WorldVec2)>) -> Self {
        self.assigned = places.into_iter().collect();
        self
    }

    pub fn place(
        &self,
        spawns: &[SpawnZone],
        uuid: SquadUuid,
        occupied: &[WorldVec2],
    ) -> WorldVec2 {
        // If placer already know a place for the squad, us it
        // (only if it is in an owned spawn zone)
        if let Some(place) = self.assigned.get(&uuid)
            && spawns.iter().any(|s| s.contains(*place))
        {
            return *place;
        }

        // Elsewhere, pick randomly a free place in available spawn zone
        let candidates: Vec<&WorldVec2> = self
            .places
            .iter()
            .filter(|p| !occupied.contains(p) && spawns.iter().any(|s| s.contains(**p)))
            .collect();
        if let Some(place) = fastrand::choice(candidates) {
            return *place;
        }

        // No free place: use the start of an owned spawn zone
        spawns
            .first()
            .map(|s| s.start)
            .expect("Side must own at least one spawn zone")
    }

    pub fn orders(
        &self,
        _w: &oc_root::WorldConfig,
        _mod_: &oc_mod::Mod,
        snapshot: &oc_world::snapshot::Snapshot,
        squad: &oc_individual::squad::Squad,
    ) -> Vec<oc_individual::order::Order> {
        let policy = &self.orders[squad.side];

        if matches!(policy, OrderPolicy::None) {
            return vec![];
        }

        let opposite_side = squad.side.opposite();
        let nearest_enemy = snapshot
            .individuals
            .iter()
            .filter(|individual| individual.side == opposite_side)
            .min_by(|a, b| {
                let da = WorldVec2::from(a.position) - squad.position;
                let db = WorldVec2::from(b.position) - squad.position;
                (da.x * da.x + da.y * da.y).total_cmp(&(db.x * db.x + db.y * db.y))
            });

        let direction = match nearest_enemy {
            Some(individual) => {
                Direction::from_points2d(squad.position.into(), individual.position.into())
            }
            None => oc_utils::d2::Direction::default(),
        };

        vec![match policy {
            OrderPolicy::None => unreachable!(),
            OrderPolicy::Hide => oc_individual::order::Order::Hide(direction),
            OrderPolicy::Defend => oc_individual::order::Order::Defend(direction),
        }]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde_derive::Deserialize)]
pub enum OrderPolicy {
    None,
    #[default]
    Hide,
    Defend,
}

impl OrderPolicy {
    /// Name of the policy as command line argument value (like `--order-a hide`)
    pub fn arg_name(&self) -> &'static str {
        match self {
            OrderPolicy::None => "none",
            OrderPolicy::Hide => "hide",
            OrderPolicy::Defend => "defend",
        }
    }
}
