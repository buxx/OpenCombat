use bevy::prelude::*;
use oc_individual::behavior;

#[derive(Debug, Component)]
pub struct IndividualIndex(pub oc_individual::IndividualIndex);

impl AsRef<oc_individual::IndividualIndex> for IndividualIndex {
    fn as_ref(&self) -> &oc_individual::IndividualIndex {
        &self.0
    }
}

// FIXME: organize (Components also in crates/oc_battle_gui/src/ingame/individual.rs)
#[derive(Debug, Component)]
pub struct Side(pub oc_root::side::Side);

// FIXME: organize (Components also in crates/oc_battle_gui/src/ingame/individual.rs)
#[derive(Debug, Component)]
pub struct Behavior(pub behavior::Behavior);

// FIXME: organize (Components also in crates/oc_battle_gui/src/ingame/individual.rs)
#[derive(Debug, Component)]
pub struct Intent(pub behavior::Intent);

// FIXME: organize (Components also in crates/oc_battle_gui/src/ingame/individual.rs)
#[derive(Debug, Component)]
pub struct Orders(pub Vec<oc_individual::order::Order>);
