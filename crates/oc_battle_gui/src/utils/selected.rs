use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    ingame::{InGameState, input::left_click::LeftClickModeType},
    states::{AppState, PointerIn},
};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct SelectedGizmos;

#[derive(Debug, Component, Default, Deref, DerefMut)]
pub struct Selected(pub bool);

pub fn setup<T: Selection + Send + Sync + 'static>(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "utils-selected-setup");
    let (gizmos, _) = config.config_mut::<SelectedGizmos>();
    gizmos.line.width = 1.0;
}

pub fn draw<T: Selection + Send + Sync + 'static>(
    mut gizmos: Gizmos<SelectedGizmos>,
    q: Query<(&Transform, &Selected)>, // FIXME: need I (identifier to not select other types like vehicle)
) {
    for (transform, selected) in &q {
        if selected.0 {
            let size = T::size();
            let isometry = Isometry2d::from_translation(transform.translation.truncate());
            gizmos.rect_2d(isometry, size, T::color());
        }
    }
}

/// Un select all. Observer which select must stop propagation to avoid execute this observer.
// `T` is unused in the body but keeps `unselect::<T>` a distinct Bevy system per `SelectedPlugin<T>`.
#[allow(clippy::extra_unused_type_parameters)]
fn unselect<T: Selection + Send + Sync + 'static>(
    click: On<PointerClick>,
    mut query: Query<&mut Selected>,
) {
    if click.button == PointerButton::Primary {
        query.iter_mut().for_each(|mut selected| selected.0 = false);
    }
}

#[derive(Debug, Default)]
pub struct SelectedPlugin<T: Selection + Send + Sync + 'static>(PhantomData<T>);

impl<T: Selection + Send + Sync + 'static> Plugin for SelectedPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<SelectedGizmos>()
            .add_systems(Startup, setup::<T>)
            .add_systems(Update, draw::<T>)
            .add_observer(
                unselect::<T>
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Select))
                    .run_if(in_state(PointerIn::Battle)),
            );
    }
}

pub trait Selection {
    fn size() -> Vec2;
    fn color() -> Srgba;
}
