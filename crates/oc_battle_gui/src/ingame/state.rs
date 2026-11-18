use std::time::Instant;

use bevy::prelude::*;
use derive_more::Constructor;
use oc_individual::{IndividualIndex, order::Order, squad::SquadIndex};
use oc_root::geo::WorldVec3;

// TODO: use getset and remove collisions pub
#[derive(Debug, Resource, Default, Clone)]
pub struct State {
    /// Squad under selection
    selected_squads: Vec<SquadIndex>,
    /// Individuals of selected squads
    selected_squads_individuals: Vec<IndividualIndex>,
    /// Individuals specially selected
    selected_individuals: Vec<IndividualIndex>,
    /// Orders which player is creating
    pending_orders: Vec<Order>,
    pub collisions: Vec<(Instant, WorldVec3)>,
}

impl State {
    pub fn selected_squads(&self) -> &[SquadIndex] {
        &self.selected_squads
    }

    #[allow(unused)]
    pub fn selected_squads_individuals(&self) -> &[IndividualIndex] {
        &self.selected_squads_individuals
    }

    #[allow(unused)]
    pub fn selected_individuals(&self) -> &[IndividualIndex] {
        &self.selected_individuals
    }

    /// Entrypoint for selection update. Only way to update selection which require set all values at once.
    #[must_use]
    pub fn update_selected(
        &mut self,
        squads: Vec<SquadIndex>,
        squads_individuals: Vec<IndividualIndex>,
        individuals: Vec<IndividualIndex>,
    ) -> SelectionUpdated {
        self.selected_squads = squads;
        self.selected_squads_individuals = squads_individuals;
        self.selected_individuals = individuals;

        SelectionUpdated::new(
            self.selected_squads.clone(),
            self.selected_squads_individuals.clone(),
            self.selected_individuals.clone(),
        )
    }

    pub fn pending_orders(&self) -> &[Order] {
        &self.pending_orders
    }

    pub fn clear_pending_orders(&mut self) {
        self.pending_orders.clear()
    }

    pub fn push_pending_orders(&mut self, value: Order) {
        self.pending_orders.push(value)
    }

    pub fn pop_pending_order(&mut self) -> Option<Order> {
        self.pending_orders.pop()
    }

    pub fn selection(&self) -> Selection {
        Selection {
            selected_squads: self.selected_squads.clone(),
            selected_squads_individuals: self.selected_squads_individuals.clone(),
            selected_individuals: self.selected_individuals.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub selected_squads: Vec<SquadIndex>,
    pub selected_squads_individuals: Vec<IndividualIndex>,
    pub selected_individuals: Vec<IndividualIndex>,
}

#[derive(Debug, Event, Constructor)]
pub struct SelectionUpdated {
    pub selected_squads: Vec<SquadIndex>,
    #[allow(unused)]
    pub selected_squads_individuals: Vec<IndividualIndex>,
    #[allow(unused)]
    pub selected_individuals: Vec<IndividualIndex>,
}
