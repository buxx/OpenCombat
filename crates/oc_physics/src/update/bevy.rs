use std::hash::Hash;

use bevy::prelude::*;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_mod::nature::Traversability;
use oc_root::{
    Wcfg, WcfgFrom,
    geo::{ScreenVec2, WorldVec3},
};
use oc_utils::{bevy::EntityMapping, let_ok, let_some};

#[derive(Debug, Event)]
pub struct SetPositionEvent<I>(pub I, pub WorldVec3, pub WorldVec3); // new, before

#[derive(Debug, Event)]
pub struct SetTileEvent<I>(pub I, pub WorldTileIndex, pub WorldTileIndex); // new, before

#[derive(Debug, Event)]
pub struct SetRegionEvent<I>(pub I, pub WorldRegionIndex, pub WorldRegionIndex); // new, before

#[derive(Debug, Event)]
pub struct PushForceEvent<I>(pub I, pub crate::Force);

#[derive(Debug, Event)]
pub struct RemoveForceEvent<I>(pub I, pub crate::Force);

#[derive(Debug, Event)]
pub struct SetVolumesEvent<I>(
    pub I,
    pub Vec<(crate::volume::Volume, Traversability)>,
    pub Vec<(crate::volume::Volume, Traversability)>,
); // new, before

#[derive(Debug, Component)]
pub struct Position(pub WorldVec3);

#[derive(Debug, Component)]
pub struct Tile(pub WorldTileIndex);

#[derive(Debug, Component)]
pub struct Region(pub WorldRegionIndex);

#[derive(Debug, Component)]
pub struct Forces(pub Vec<crate::Force>);

#[derive(Debug, Component)]
pub struct Volumes(pub Vec<(crate::volume::Volume, Traversability)>);

pub trait UpdatePhysicsEvent<I> {
    fn i(&self) -> I;
    fn value(&self) -> &super::Update;
}

pub fn on_update_physics<
    I: Send + Sync + std::fmt::Debug + 'static,
    E: Event + UpdatePhysicsEvent<I>,
>(
    update: On<E>,
    mut commands: Commands,
) {
    let (i, update) = (update.i(), &update.value());

    match update {
        super::Update::SetPosition(position2, position1) => {
            commands.trigger(SetPositionEvent(i, *position2, *position1));
        }
        super::Update::SetTile(tile2, tile1) => {
            commands.trigger(SetTileEvent(i, *tile2, *tile1));
        }
        super::Update::SetRegion(region2, region1) => {
            commands.trigger(SetRegionEvent(i, *region2, *region1));
        }
        super::Update::PushForce(force) => {
            commands.trigger(PushForceEvent(i, force.clone()));
        }
        super::Update::RemoveForce(force) => {
            commands.trigger(RemoveForceEvent(i, force.clone()));
        }
        super::Update::SetVolumes(volumes2, volumes1) => {
            commands.trigger(SetVolumesEvent(i, volumes2.clone(), volumes1.clone()));
        }
    }
}

fn on_set_position_event<I: Hash + Eq + Send + Sync + std::fmt::Debug + 'static>(
    event: On<SetPositionEvent<I>>,
    mut query: Query<(&mut Position, &mut Transform)>,
    state: Res<EntityMapping<I>>,
    w: Res<Wcfg>,
) {
    let_some!(w = &w.0, return);
    let_some!(entity = state.get(&event.0), return);
    let Ok((mut position_, mut transform)) = query.get_mut(*entity) else {
        return;
    };
    position_.0 = event.1;
    let translation = ScreenVec2::from_(event.1, w);
    transform.translation.x = translation.x;
    transform.translation.y = translation.y;
}

fn on_set_tile_event<I: Hash + Eq + Send + Sync + 'static>(
    event: On<SetTileEvent<I>>,
    mut query: Query<&mut Tile>,
    state: Res<EntityMapping<I>>,
) {
    let_some!(entity = state.get(&event.0), return);
    let_ok!(mut component = query.get_mut(*entity), return);
    // tracing::trace!(name="update-individual-tile", i=?tile.0, tile=?tile.1);

    component.0 = event.1;
}

fn on_set_region_event<I: Hash + Eq + Send + Sync + 'static>(
    event: On<SetRegionEvent<I>>,
    mut query: Query<&mut Region>,
    state: Res<EntityMapping<I>>,
) {
    let_some!(entity = state.get(&event.0), return);
    let_ok!(mut component = query.get_mut(*entity), return);
    // tracing::trace!(name="update-individual-region", i=?region.0, region=?region.1);

    component.0 = event.1;
}

fn on_push_force_event<I: Hash + Eq + Send + Sync + 'static>(
    event: On<PushForceEvent<I>>,
    mut query: Query<&mut Forces>,
    state: Res<EntityMapping<I>>,
) {
    let_some!(entity = state.get(&event.0), return);
    let_ok!(mut component = query.get_mut(*entity), return);
    // tracing::trace!(name = "update-individual-force-push", i=?force.0, force=?force.1);

    component.0.push(event.1.clone());
}

fn on_remove_force_event<I: Hash + Eq + Send + Sync + 'static>(
    event: On<RemoveForceEvent<I>>,
    mut query: Query<&mut Forces>,
    state: Res<EntityMapping<I>>,
) {
    let_some!(entity = state.get(&event.0), return);
    let_ok!(mut component = query.get_mut(*entity), return);
    // tracing::trace!(name = "update-individual-force-remove", i=?force.0, force=?force.1);

    component.0.retain(|f| f != &event.1);
}

fn on_set_volume<I: Hash + Eq + Send + Sync + 'static>(
    event: On<SetVolumesEvent<I>>,
    mut query: Query<&mut Volumes>,
    state: Res<EntityMapping<I>>,
) {
    let_some!(entity = state.get(&event.0), return);
    let_ok!(mut component = query.get_mut(*entity), return);
    // tracing::trace!(name="update-individual-region", i=?region.0, region=?region.1);

    component.0 = event.1.clone();
}

#[derive(Debug)]
pub struct PhysicsPlugin<I: Hash + Eq + Send + Sync + 'static, E: Event + UpdatePhysicsEvent<I>> {
    _marker: std::marker::PhantomData<(I, E)>,
}

impl<I: Hash + Eq + Send + Sync + 'static, E: Event + UpdatePhysicsEvent<I>> Default
    for PhysicsPlugin<I, E>
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<I: Hash + Eq + Send + Sync + std::fmt::Debug + 'static, E: Event + UpdatePhysicsEvent<I>>
    Plugin for PhysicsPlugin<I, E>
{
    fn build(&self, app: &mut App) {
        app.add_observer(on_update_physics::<I, E>)
            .add_observer(on_set_position_event::<I>)
            .add_observer(on_set_tile_event::<I>)
            .add_observer(on_set_region_event::<I>)
            .add_observer(on_push_force_event::<I>)
            .add_observer(on_set_volume::<I>)
            .add_observer(on_remove_force_event::<I>);
    }
}
