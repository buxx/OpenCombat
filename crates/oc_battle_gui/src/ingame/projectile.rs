use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use oc_geo::region::{Region as _, WorldRegionIndex};
use oc_physics::Physic;
use oc_physics::collision::Material_;
use oc_physics::update::bevy::{Forces, PhysicsPlugin, Position, Region, Tile, Volumes};
use oc_root::WcfgFrom;
use oc_root::geo::ScreenVec2;
use oc_root::y::Y;
use oc_utils::bevy::EntityMapping;
use oc_utils::{let_ok, let_some, true_};

use crate::entity::projectile::ProjectileId;
use crate::ingame;
use crate::ingame::draw::Z_PROJECTILE;
use crate::ingame::input::individual::UpdateProjectilePhysicsEvent;
use crate::ingame::input::projectile::InsertProjectileEvent;
use crate::ingame::physics::{Direction, IgnoreOrigin, PhysicsUpdatedPosition};
use crate::ingame::region::ForgottenRegion;
use crate::states::{AppState, GameConfig};

#[derive(Debug, Deref, Event)]
pub struct ForgotProjectile(pub oc_projectile::ProjectileId);

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugin::<
            oc_projectile::ProjectileId,
            UpdateProjectilePhysicsEvent,
        >::default())
            .add_observer(on_insert_projectile)
            .init_resource::<EntityMapping<oc_projectile::ProjectileId>>()
            .add_observer(on_update_position)
            .add_observer(on_forgotten_region)
            .add_observer(on_forgot_projectile)
            .add_observer(on_battle_end)
            .add_systems(
                Update,
                ingame::physics::physics_step::<oc_projectile::ProjectileId, ProjectileId>
                    .run_if(in_state(AppState::InGame))
                    .run_if(not(in_state(ingame::BattlePhase::End))),
            );
    }
}

pub fn on_insert_projectile(
    projectile: On<InsertProjectileEvent>,
    mut commands: Commands,
    g: Res<GameConfig>,
    mut state: ResMut<EntityMapping<oc_projectile::ProjectileId>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let_some!(g = &g.0, return);
    tracing::trace!(name="spawn-projectile", i=?projectile.0, position=?projectile.1.position(), forces=?projectile.1.forces(&g.w));

    let position = projectile.1.position();
    let position_ = ScreenVec2::from_(position, &g.w);
    let line = Polyline2d::new(vec![position_.into()]);
    let volumes = projectile
        .1
        .volumes(position, false, &g.w, &g.mod_)
        .clone()
        .into_iter()
        .map(|(p, v, _)| (p, v))
        .collect::<Vec<_>>();

    let entity = commands
        .spawn((
            ProjectileId(projectile.0),
            Position(position),
            Tile(projectile.1.tile()),
            Region(projectile.1.region()),
            Forces(projectile.1.forces(&g.w).clone()),
            Material_(Some(oc_root::material::MaterialKind::Projectile)),
            Volumes(volumes),
            Direction(oc_utils::d2::Direction::NORTH),
            IgnoreOrigin(projectile.1.collision_ignore_origin().unwrap_or(position)),
            Mesh2d(meshes.add(line)),
            MeshMaterial2d(materials.add(Color::from(RED))),
            Transform::from_xyz(position_.x, position_.y, Z_PROJECTILE),
        ))
        .id();
    state.insert(projectile.0, entity);
}

fn on_update_position(
    event: On<PhysicsUpdatedPosition<oc_projectile::ProjectileId>>,
    g: Res<GameConfig>,
    projectiles: Res<EntityMapping<oc_projectile::ProjectileId>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&Mesh2d>,
) {
    let_some!(g = &g.0, return);
    let (i, position, previous) = (event.0, event.1, event.2);
    let_some!(entity = projectiles.get(&i), return);
    let_ok!(mesh = query.get(*entity), return);
    let_some!(mut mesh = meshes.get_mut(mesh), return);

    let relative = previous.to_gui_y(&g.w) - position.to_gui_y(&g.w);
    *mesh = Polyline2d::new(vec![Vec2::new(0.0, 0.0), Vec2::new(relative.x, relative.y)]).into();
}

// TODO: should be automatized (macro? derive ?)
fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    query: Query<(&Region, &ProjectileId)>,
) {
    for (region_, i) in query {
        let region_: WorldRegionIndex = region_.0;
        if region_ == region.0 {
            tracing::trace!(name = "trigger-forgot-projectile", i=?i);
            commands.trigger(ForgotProjectile(i.0));
        }
    }
}

/// Battle is ended: no more projectiles
fn on_battle_end(
    phase: On<ingame::SetBattlePhase>,
    projectiles: Query<&ProjectileId>,
    mut commands: Commands,
) {
    true_!(phase.phase() == ingame::BattlePhase::End, return);

    for projectile in projectiles {
        commands.trigger(ForgotProjectile(projectile.0));
    }
}

pub fn on_forgot_projectile(
    projectile: On<ForgotProjectile>,
    mut commands: Commands,
    mut projectiles: ResMut<EntityMapping<oc_projectile::ProjectileId>>,
) {
    if let Some(entity) = projectiles.remove(&projectile.0) {
        tracing::trace!(name = "remove-projectile", i=?projectile.0);
        commands.entity(entity).despawn();
    }
}
