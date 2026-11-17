//! Choose flags to capture: squads are spread over flags not controlled by the side (neutral,
//! enemy or contested ones) instead of all going to the nearest one. For a contested flag, squads
//! go to the nearest enemy inside it (AI cheats: these enemies may not be seen).
use oc_individual::order::Order;
use oc_root::geo::WorldVec2;

use crate::perception::{FlagView, Perception, SquadView};

/// Extra cost (meters) of a flag for each squad already sent to it
pub const SPREAD_PENALTY: f32 = 150.0;
/// Cost reduction (meters) of the flag a squad is already going to (avoid target changes)
pub const KEEP_BONUS: f32 = 50.0;

/// Destination to capture a flag for each given squad (same order), None if all flags are
/// controlled. Each squad takes the cheapest flag (see [`cost`]), given squads sent before it.
pub fn targets(perception: &Perception, squads: &[&SquadView]) -> Vec<Option<WorldVec2>> {
    let flags = flags_to_capture(perception);
    let mut sent = vec![0usize; flags.len()];

    squads
        .iter()
        .map(|squad| {
            let cost = |i: usize| cost(perception, &flags, squad, i, sent[i]);
            let best = (0..flags.len()).min_by(|a, b| cost(*a).total_cmp(&cost(*b)))?;
            sent[best] += 1;
            Some(destination(perception, squad, flags[best]))
        })
        .collect()
}

/// Flags not controlled by the side
fn flags_to_capture(perception: &Perception) -> Vec<&FlagView> {
    perception
        .flags
        .iter()
        .filter(|f| !f.controlled_by(perception.side))
        .collect()
}

/// Where squad must go to capture flag: nearest enemy inside it when contested, else its center
fn destination(perception: &Perception, squad: &SquadView, flag: &FlagView) -> WorldVec2 {
    if !flag.contested() {
        return flag.center;
    }

    perception
        .nearest(squad.position, flag.enemies.iter().copied(), |p| *p)
        .unwrap_or(flag.center)
}

/// Cost (meters) for squad to capture `flags[i]`, already targeted by `sent` squads: distance,
/// plus [`SPREAD_PENALTY`] by sent squad, minus [`KEEP_BONUS`] if squad already goes there
fn cost(
    perception: &Perception,
    flags: &[&FlagView],
    squad: &SquadView,
    i: usize,
    sent: usize,
) -> f32 {
    let mut cost =
        perception.meters(squad.position, flags[i].center) + SPREAD_PENALTY * sent as f32;
    if going_to(perception, flags, squad) == Some(i) {
        cost -= KEEP_BONUS;
    }
    cost
}

/// Index of the flag (in `flags`) squad currently moves to: the nearest one of its move
/// destination
fn going_to(perception: &Perception, flags: &[&FlagView], squad: &SquadView) -> Option<usize> {
    let Some(Order::MoveTo(destination)) = squad.orders.first() else {
        return None;
    };

    perception.nearest(*destination, 0..flags.len(), |i| flags[*i].center)
}
