use bevy::prelude::*;
use oc_individual::order::OrderType;
use oc_mod::grenade::GrenadeNature;
use oc_network::GameConfig;
use oc_root::geo::WorldVec2;
use oc_utils::let_some;
use strum_macros::{Display, EnumIter};

use crate::{
    ingame::{
        draw::UI_FILE,
        input::left_click::{LeftClickMode, SetLeftClick},
    },
    menu::contextual::{self, ContextualMenu, open::OpenContextualMenu},
};

const ITEMS_IDLE_START_X: f32 = 0.;
const ITEMS_IDLE_START_Y: f32 = 0.;
const ITEMS_HOVER_START_X: f32 = 71.;
const ITEMS_HOVER_START_Y: f32 = 0.;
const ITEMS_DISABLED_START_X: f32 = 142.;
const ITEM_HEIGHT: f32 = 15.;
const ITEM_WIDTH: f32 = 70.;

#[derive(Debug, Event)]
pub struct PrepareOpenSquadContextualMenu(pub WorldVec2);

#[derive(Debug, Event)]
pub struct Open {
    pub position: WorldVec2,
    pub frag: bool,
    pub smoke: bool,
}

#[derive(Debug)]
pub struct Menu;

#[derive(Debug, Clone, Copy, Event, Component, EnumIter, Display)]
pub enum Choice {
    Move,
    MoveFast,
    Sneak,
    Engage,
    Suppress,
    Grenade,
    Smoke,
    Defend,
    Hide,
}

impl OpenContextualMenu<Menu> for Open {
    fn position(&self) -> WorldVec2 {
        self.position
    }

    fn is_disabled(&self, choice: &Choice) -> bool {
        match choice {
            Choice::Grenade => !self.frag,
            Choice::Smoke => !self.smoke,
            _ => false,
        }
    }
}

impl ContextualMenu for Menu {
    type OpenEvent = Open;
    type ChoiceEvent = Choice;
    type Choices = Choice;

    fn image() -> &'static str {
        UI_FILE
    }
}

impl contextual::choice::Choice<Choice> for Choice {
    fn name(&self) -> String {
        self.to_string()
    }

    fn event(&self) -> Choice {
        *self
    }

    fn idle(&self) -> Rect {
        item_rect(*self, ITEMS_IDLE_START_X, ITEMS_IDLE_START_Y)
    }

    fn hover(&self) -> Rect {
        item_rect(*self, ITEMS_HOVER_START_X, ITEMS_HOVER_START_Y)
    }

    fn disabled(&self) -> Rect {
        item_rect(*self, ITEMS_DISABLED_START_X, ITEMS_IDLE_START_Y)
    }
}

fn item_rect(choice: Choice, start_x: f32, start_y: f32) -> Rect {
    let index = match choice {
        Choice::Move => 0,
        Choice::MoveFast => 1,
        Choice::Sneak => 2,
        Choice::Engage => 3,
        Choice::Suppress => 4,
        Choice::Grenade => 5,
        Choice::Smoke => 6,
        Choice::Defend => 7,
        Choice::Hide => 8,
    };
    let y0 = start_y + index as f32 * ITEM_HEIGHT;
    Rect::new(start_x, y0, start_x + ITEM_WIDTH, y0 + ITEM_HEIGHT)
}

pub fn on_prepare_open_squad_contextual_menu(
    event: On<PrepareOpenSquadContextualMenu>,
    state: Res<crate::ingame::state::State>,
    world: Res<crate::world::World>,
    g: Res<crate::states::GameConfig>,
    mut commands: Commands,
) {
    let_some!(g = &g.0, return);

    // Squad orders require selected squads owning a leader able to lead
    if !state.selected_squads().iter().any(|i| {
        world
            .squad(*i)
            .and_then(|squad| world.individual(squad.leader()))
            .is_some_and(|leader| leader.can_leadership())
    }) {
        tracing::debug!("No selected squad with a leader: don't open contextual menu");
        return;
    }

    tracing::debug!("Trigger open contextual menu");
    let (frag, smoke) = grenades(state, world, g);

    commands.trigger(Open {
        position: event.0,
        frag,
        smoke,
    })
}

fn grenades(
    state: Res<'_, crate::ingame::state::State>,
    world: Res<'_, crate::world::World>,
    g: &GameConfig,
) -> (bool, bool) {
    let (mut frag, mut smoke) = (false, false);

    for grenade in state
        .selected_squads()
        .iter()
        .filter_map(|i| world.squad(*i))
        .flat_map(|squad| squad.members.iter())
        .filter_map(|i| world.individual(*i))
        .flat_map(|individual| individual.weapons.grenades.iter())
    {
        match g.mod_.grenade(*grenade).nature {
            GrenadeNature::Frag => frag = true,
            GrenadeNature::Smoke => smoke = true,
        }
    }

    (frag, smoke)
}

pub fn on_choose(item: On<Choice>, mut commands: Commands) {
    let order = match *item {
        Choice::Move => OrderType::MoveTo,
        Choice::MoveFast => OrderType::MoveFastTo,
        Choice::Sneak => OrderType::SneakTo,
        Choice::Engage => OrderType::Engage,
        Choice::Suppress => OrderType::Suppress,
        Choice::Grenade => OrderType::Grenade,
        Choice::Smoke => OrderType::Smoke,
        Choice::Defend => OrderType::Defend,
        Choice::Hide => OrderType::Hide,
    };

    commands.trigger(SetLeftClick(LeftClickMode::Order(order)));
}
