use bevy::prelude::*;
use oc_utils::let_some;

use std::time::Duration;
#[derive(Debug, Event)]
pub struct UpdateRemaining(pub Duration);

#[derive(Component, Default, Clone)]
pub struct RemainingText;

pub fn remaining() -> impl Scene {
    bsn! {
        RemainingText
        Text("--:--")
        TextFont { font_size: px(12) }
        Pickable::IGNORE
    }
}

/// Last remaining time received from server, anticipated by the GUI between two server updates
#[derive(Debug, Default, Resource)]
pub struct Remaining {
    left: Option<Duration>,
    fresh: bool,
}

pub fn on_update_remaining(event: On<UpdateRemaining>, mut remaining: ResMut<Remaining>) {
    remaining.left = Some(event.0);
    remaining.fresh = true;
}

pub fn tick_remaining(
    time: Res<Time>,
    mut remaining: ResMut<Remaining>,
    mut text: Single<&mut Text, With<RemainingText>>,
    mut timer: Local<Option<Timer>>,
) {
    let_some!(left = remaining.left, return);
    let timer =
        timer.get_or_insert_with(|| Timer::new(Duration::from_secs(1), TimerMode::Repeating));

    if remaining.fresh {
        remaining.fresh = false;
        // Fresh server value: display it right now and restart the second
        timer.reset();
    } else {
        remaining.left = Some(left.saturating_sub(time.delta()));
        if !timer.tick(time.delta()).just_finished() {
            return;
        }
    }

    let secs = remaining.left.unwrap_or_default().as_secs();
    text.0 = format!("{:02}:{:02}", secs / 60, secs % 60);
}
