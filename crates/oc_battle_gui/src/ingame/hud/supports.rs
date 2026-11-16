use crate::ingame::{
    hud::GAP,
    input::left_click::{LeftClickMode, SetLeftClick, support::RequestedSupports},
};
use crate::world::World;
use bevy::prelude::*;
use oc_utils::{let_ok, true_};
use oc_world::support::SupportType;

const BUTTON_BG: Color = Color::srgb(0.25, 0.25, 0.25);
const BUTTON_BG_HOVER: Color = Color::srgb(0.4, 0.4, 0.4);
const BUTTON_BG_DISABLED: Color = Color::srgb(0.15, 0.15, 0.15);
const TEXT_DISABLED: Color = Color::srgb(0.45, 0.45, 0.45);

#[derive(Component, Default, Clone)]
pub struct SupportList;

#[derive(Debug, Event)]
pub struct SetSupportCountEvent(pub SupportType, pub u8);

#[derive(Debug, Event)]
pub struct SupportExecutedEvent(pub SupportType);

#[derive(Component)]
pub struct SupportButton {
    pub type_: SupportType,
    pub count: u8,
}

impl SupportButton {
    pub fn available(&self) -> bool {
        self.count > 0
    }
}

fn support_label(type_: &SupportType, count: u8) -> String {
    format!("{} ({count})", type_.name())
}

pub fn supports() -> impl Scene {
    bsn! {
        SupportList
        Node {
            flex_shrink: 0.,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(GAP),
            column_gap: px(GAP),
        }
    }
}

fn support_button(support: (SupportType, u8)) -> impl Scene {
    let (type_, count) = support;
    let label = support_label(&type_, count);
    let (bg, text) = button_colors(count > 0);

    bsn! {
        Node {
            padding: UiRect::axes(px(12), px(6)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor({bg})
        on(on_hover)
        on(on_out)
        on(on_click)
        Children [
            Text({label})
            TextFont { font_size: px(12) }
            TextColor({text})
            Pickable::IGNORE
        ]
    }
}

pub fn spawn_support_buttons(
    mut commands: Commands,
    world: Res<World>,
    list: Single<Entity, With<SupportList>>,
    buttons: Query<(), With<SupportButton>>,
) {
    let list = list.into_inner();
    true_!(buttons.is_empty() && !world.supports.is_empty(), return);

    for (type_, count) in &world.supports {
        commands
            .spawn_scene(support_button((type_.clone(), *count)))
            .insert((
                SupportButton {
                    type_: type_.clone(),
                    count: *count,
                },
                ChildOf(list),
            ));
    }
}

fn on_hover(e: On<PointerOver>, mut q: Query<(&SupportButton, &mut BackgroundColor)>) {
    if let Ok((button, mut bg)) = q.get_mut(e.entity)
        && button.available()
    {
        bg.0 = BUTTON_BG_HOVER;
    }
}

fn on_out(e: On<PointerOut>, mut q: Query<(&SupportButton, &mut BackgroundColor)>) {
    if let Ok((button, mut bg)) = q.get_mut(e.entity)
        && button.available()
    {
        bg.0 = BUTTON_BG;
    }
}

fn on_click(mut e: On<PointerClick>, mut commands: Commands, buttons: Query<&SupportButton>) {
    let_ok!(button = buttons.get(e.entity), return);
    e.propagate(false);
    true_!(button.available(), return);
    commands.trigger(SetLeftClick(LeftClickMode::Support(button.type_.clone())));
}

pub fn on_support_executed(
    event: On<SupportExecutedEvent>,
    mut requested: ResMut<RequestedSupports>,
) {
    requested.remove(&event.0);
}

pub fn on_set_support_count(
    event: On<SetSupportCountEvent>,
    mut world: ResMut<World>,
    mut buttons: Query<(&mut SupportButton, &mut BackgroundColor, &Children)>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
    mut requested: ResMut<RequestedSupports>,
) {
    let SetSupportCountEvent(type_, count) = &*event;
    let count = *count;

    if count == 0 {
        requested.remove(type_);
    }

    for (t, c) in world.supports.iter_mut() {
        if t == type_ {
            *c = count;
        }
    }

    let (bg_color, text_color) = button_colors(count > 0);
    for (mut button, mut bg, children) in &mut buttons {
        if &button.type_ != type_ {
            continue;
        }
        button.count = count;
        bg.0 = bg_color;
        for child in children {
            if let Ok((mut text, mut color)) = texts.get_mut(*child) {
                color.0 = text_color;
                text.0 = support_label(type_, count);
            }
        }
    }
}

fn button_colors(available: bool) -> (Color, Color) {
    if available {
        (BUTTON_BG, Color::WHITE)
    } else {
        (BUTTON_BG_DISABLED, TEXT_DISABLED)
    }
}
