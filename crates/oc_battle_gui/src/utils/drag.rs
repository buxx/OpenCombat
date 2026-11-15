use std::marker::PhantomData;

use bevy::prelude::*;
use oc_root::{
    WcfgFrom,
    geo::{ScreenVec2, WorldVec2},
    y::V,
};
use oc_utils::{d2::Direction, let_ok, let_some};

use crate::{cursor_to, states::GameConfig, utils::selected};

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct Cursor(pub Option<(WorldVec2, bool)>); // Start dragging position of cursor; bool = entity under cursor was already selected

#[derive(Debug, Component)]
pub struct Dragged<T: Dragging + std::fmt::Debug + Send + Sync + 'static>(PhantomData<T>, bool);

#[derive(Debug, Component, Clone, Copy)]
pub struct Phantom(pub Entity); // bool = was selected

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static> Default for Dragged<T> {
    fn default() -> Self {
        Self(PhantomData, false)
    }
}

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static> std::ops::Deref for Dragged<T> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static> std::ops::DerefMut for Dragged<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

pub enum Visual {
    Offset,
    Direction,
}

#[derive(Debug)]
pub struct DragPlugin<T: Dragging + std::fmt::Debug + Send + Sync + 'static, S: States + Copy> {
    in_state: Option<S>,
    _marker: PhantomData<T>,
}

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static, S: States + Copy> Default
    for DragPlugin<T, S>
{
    fn default() -> Self {
        Self {
            in_state: None,
            _marker: PhantomData,
        }
    }
}

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static, S: States + Copy> DragPlugin<T, S> {
    pub fn new(state: S) -> Self {
        Self {
            in_state: Some(state),
            _marker: PhantomData,
        }
    }
}

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static, S: States + Copy> Plugin
    for DragPlugin<T, S>
{
    fn build(&self, app: &mut App) {
        app.init_resource::<Cursor>()
            .add_observer(on_drag_stop::<T>);

        match self.in_state {
            Some(state) => match T::visual() {
                Visual::Offset => {
                    app.add_systems(Update, update_positions::<T>.run_if(in_state(state)))
                }
                Visual::Direction => {
                    app.add_systems(Update, update_directions::<T>.run_if(in_state(state)))
                }
            },
            None => match T::visual() {
                Visual::Offset => app.add_systems(Update, update_positions::<T>),
                Visual::Direction => app.add_systems(Update, update_directions::<T>),
            },
        };
    }
}

pub trait Dragging {
    /// Implementation must spawn bundle (or trigger event which spawn bundle) owning the given marker
    fn spawn(commands: &mut Commands, marker: Phantom);
    /// Implementation assume consequences of drop. `offset` is the cursor move since drag start,
    /// allowing implementations to preserve relative positions of multiple dragged entities.
    fn drop(commands: &mut Commands, subject: Entity, point: WorldVec2, offset: WorldVec2);
    /// Which visual method to apply
    fn visual() -> Visual;
}

/// Observer to attach on spawned bundle owning `Selected` + `Dragged` components.
pub fn on_drag_start<T>(
    mut event: On<PointerDragStart>,
    mut commands: Commands,
    mut selected: Query<(&Dragged<T>, &mut selected::Selected, Entity)>,
    mut cursor: ResMut<Cursor>,
) where
    T: Dragging + std::fmt::Debug + Send + Sync + 'static,
{
    let_some!(point = event.hit.position, return);
    let point = WorldVec2::new(point.x, point.y);
    tracing::trace!(name="utils-drag-on-drag-start", point=?point);

    // Consider dragged entity as selected
    let target = event.event_target();
    let_ok!((_, mut selected_, _) = selected.get_mut(target), return);
    let was_selected = selected_.0;
    selected_.0 = true;

    // Spawn phantom for each selected entities.
    // If the dragged entity was not selected, drag only it: previously selected entities must not follow.
    for (_, _, entity) in selected
        .iter()
        .filter(|(_, selected, entity)| selected.0 && (was_selected || *entity == target))
    {
        tracing::trace!(name="utils-drag-on-drag-start-spawn", point=?point, entity=?entity);
        T::spawn(&mut commands, Phantom(entity));
    }

    cursor.0 = Some((point, was_selected));
    event.propagate(false);
}

pub fn on_drag_stop<T>(
    mut event: On<PointerDragDrop>,
    mut commands: Commands,
    g: Res<GameConfig>,
    phantoms: Query<(Entity, &Phantom)>,
    mut dragged: Query<&mut selected::Selected, With<Dragged<T>>>,
    mut cursor: ResMut<Cursor>,
    camera: Single<(&Camera, &GlobalTransform)>,
) where
    T: Dragging + std::fmt::Debug + Send + Sync + 'static,
{
    let_some!(g = &g.0, return);
    tracing::trace!(name = "utils-drag-on-drag-stop");

    let point = event.pointer.position;
    let point = cursor_to!(point, camera, &g.w, WorldVec2);
    let_some!(cursor_ = cursor.0, return);
    let_ok!(mut dragged = dragged.get_mut(event.dropped), return);
    let point = WorldVec2::new(point.x, point.y);
    // Drag start cursor is stored in screen coordinates
    let start = WorldVec2::from_(ScreenVec2::new(cursor_.0.x, cursor_.0.y), &g.w);
    let offset = point - start;
    dragged.0 = cursor_.1;
    cursor.0 = None;

    for (entity, phantom) in phantoms {
        if let Visual::Offset = T::visual() {
            commands.entity(entity).despawn()
        }

        tracing::trace!(name="utils-drag-on-drag-stop-drop", point=?point, offset=?offset, entity=?entity);
        T::drop(&mut commands, phantom.0, point, offset);
    }

    event.propagate(false);
}

fn update_positions<T: Dragging + std::fmt::Debug + Send + Sync + 'static>(
    mut phantoms: Query<(&Phantom, &mut Transform), Without<Dragged<T>>>,
    cursor: Res<Cursor>,
    mut origins: Query<(&mut Dragged<T>, &mut Transform)>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    if phantoms.is_empty() {
        return;
    }

    let_some!(point = window.cursor_position(), return);
    let (camera, camera_transform) = *camera;
    let point = camera.viewport_to_world_2d(camera_transform, point);
    let_ok!(point = point, return);

    let_some!((cursor, _) = cursor.0, return);
    let cursor = Vec2::new(cursor.x, cursor.y);
    let offset = (point - cursor).extend(0.);
    let offset: Vec3 = offset;

    for (phantom, mut phantom_transform) in phantoms.iter_mut() {
        let_ok!((_, origin) = origins.get_mut(phantom.0), continue);
        phantom_transform.translation = origin.translation + offset;
        tracing::trace!(name="utils-drag-update-position", point=?point, phantom=?phantom, translation=?phantom_transform.translation);
    }
}

fn update_directions<T: Dragging + std::fmt::Debug + Send + Sync + 'static>(
    mut phantoms: Query<(&Phantom, &mut Transform), With<Dragged<T>>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    if phantoms.is_empty() {
        return;
    }

    let_some!(point = window.cursor_position(), return);
    let (camera, camera_transform) = *camera;
    let point = camera.viewport_to_world_2d(camera_transform, point);
    let_ok!(point = point, return);

    let cursor = glam::Vec2::new(point.x, point.y);

    for (phantom, mut phantom_transform) in phantoms.iter_mut() {
        let reference = phantom_transform.translation;
        let reference = glam::Vec2::new(reference.x, reference.y);
        let direction = Direction::from_points2d(reference, cursor);
        phantom_transform.rotation = direction.bquat(V::Server); // Weird, there is mistake in points natures
        tracing::trace!(name="utils-drag-update-rotation", point=?point, phantom=?phantom, rotation=?phantom_transform.rotation);
    }
}
