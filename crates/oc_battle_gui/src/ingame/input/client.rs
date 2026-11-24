use bevy::prelude::*;
use oc_individual::network::{Individual, Squad};
use oc_network::ToClient;
use oc_projectile::network::Projectile;
use oc_root::{Wcfg, files, static_::StaticSource};
use oc_vehicle::network::Vehicle;

use crate::ingame::hud::supports::{SetSupportCountEvent, SupportExecutedEvent};
use crate::world::InsertWrecks;
use crate::{
    fx::FxEvent,
    ingame::collision::CollisionEvent,
    ingame::{
        EndResumeEvent, GameConfigReceived, SetBattlePhase, WorldResumeEvent,
        individual::ForgotIndividual,
        input::{
            individual::{
                InsertIndividualEvent, UpdateIndividualEvent, UpdateIndividualPhysicsEvent,
                UpdateProjectilePhysicsEvent, UpdateSquadEvent,
            },
            projectile::InsertProjectileEvent,
            vehicle::{ForgotVehicleEvent, InsertVehicleEvent, UpdateVehicleEvent},
        },
        projectile::ForgotProjectile,
    },
    ingame::{
        flag::UpdateFlags,
        hud::{morale::UpdateMorale, remaining::UpdateRemaining},
    },
    network,
    network::input::ToClientEvent,
    states::{GameConfig, GuiFiles},
    world::{
        InsertAlteredTiles, InsertCraters, InsertDecor, InsertTiles, RemoveDecor, UpdateSmoke,
        UpdateVisibilities,
    },
};

/// Files layout of the server whose game config has been received
fn gui_files(
    roots: &files::Roots,
    config: &oc_network::GameConfig,
    connection: files::Connection,
) -> files::Files {
    let (mod_id, world_id) = (config.mod_.canonical(), config.meta.canonical());

    match (connection, &config.static_) {
        (files::Connection::Network(server), StaticSource::Remote(port)) => {
            // Downloaded sources are stored under their id (versions don't collide)
            files::Files::new(roots.clone(), &mod_id, &mod_id, &world_id, &world_id).with_remote(
                Some(files::Remote {
                    base_url: format!("http://{}:{}", server.ip(), port),
                }),
            )
        }
        // Sources are used in place, under their name
        _ => files::Files::new(
            roots.clone(),
            config.mod_.name(),
            &mod_id,
            &config.meta.name,
            &world_id,
        ),
    }
}

pub fn on_to_client(
    to_client: On<ToClientEvent>,
    mut commands: Commands,
    mut g: ResMut<GameConfig>,
    mut files: ResMut<GuiFiles>,
    network: Res<network::state::State>,
    mut w: ResMut<Wcfg>,
    app_config: Res<crate::config::Config>,
) {
    tracing::trace!(name = "ingame-input-client");

    match &to_client.0 {
        ToClient::GameConfig(config) => {
            tracing::debug!("Received GameConfig");
            g.0 = Some((**config).clone());
            w.0 = Some(config.w.clone());
            files.0 = network
                .server
                .clone()
                .map(|connect| gui_files(&app_config.roots, config, connect.into()));
            commands.trigger(GameConfigReceived);
        }
        ToClient::WorldResume(resume) => {
            commands.trigger(WorldResumeEvent(resume.clone()));
        }
        ToClient::EndResume(resume) => {
            tracing::debug!("Received EndResume: {resume:?}");
            commands.trigger(EndResumeEvent(*resume));
        }
        ToClient::Squad(message) => match message {
            Squad::Update(i, update) => {
                commands.trigger(UpdateSquadEvent(*i, update.clone()));
            }
        },
        ToClient::Individual(message) => match message {
            Individual::Insert(i, individual) => {
                commands.trigger(InsertIndividualEvent(*i, individual.clone()));
            }
            Individual::Update(i, update) => {
                commands.trigger(UpdateIndividualEvent(*i, update.clone()));
            }
            Individual::Physics(i, update) => {
                commands.trigger(UpdateIndividualPhysicsEvent(*i, update.clone()));
            }
            Individual::Forgot(i) => {
                commands.trigger(ForgotIndividual(*i));
            }
        },
        ToClient::Vehicle(message) => match message {
            Vehicle::Insert(i, vehicle) => {
                commands.trigger(InsertVehicleEvent(*i, vehicle.clone()));
            }
            Vehicle::Update(i, update) => {
                commands.trigger(UpdateVehicleEvent(*i, update.clone()));
            }
            Vehicle::Forgot(i) => {
                commands.trigger(ForgotVehicleEvent(*i));
            }
        },
        ToClient::Projectile(message) => match message {
            Projectile::Insert(id, projectile) => {
                tracing::trace!(name="ingame-input-client-projectile-insert", i=?id);
                commands.trigger(InsertProjectileEvent(*id, projectile.clone()));
            }
            Projectile::Physics(id, update) => {
                tracing::trace!(name="ingame-input-client-projectile-physics", i=?id);
                commands.trigger(UpdateProjectilePhysicsEvent(*id, update.clone()));
            }
            Projectile::Forgot(i) => {
                commands.trigger(ForgotProjectile(*i));
            }
        },
        ToClient::Tiles(region, tiles) => {
            tracing::trace!(name="ingame-input-tiles", region=?region);
            commands.trigger(InsertTiles(*region, tiles.clone()));
        }
        ToClient::AlteredTiles(region, tiles) => {
            tracing::trace!(name="ingame-input-altered-tiles", region=?region);
            commands.trigger(InsertAlteredTiles(*region, tiles.clone()));
        }
        ToClient::Decor(region, decor) => {
            tracing::trace!(name="ingame-input-decor", region=?region);
            commands.trigger(InsertDecor(*region, decor.clone()));
        }
        ToClient::RemovedDecor(region, tiles) => {
            tracing::trace!(name="ingame-input-removed-decor", region=?region);
            commands.trigger(RemoveDecor(*region, tiles.clone()));
        }
        ToClient::Craters(region, craters) => {
            tracing::trace!(name="ingame-input-craters", region=?region);
            commands.trigger(InsertCraters(*region, craters.clone()));
        }
        ToClient::Wrecks(region, wrecks) => {
            tracing::trace!(name="ingame-input-wrecks", region=?region);
            commands.trigger(InsertWrecks(*region, wrecks.clone()));
        }
        ToClient::Smoke(region, smoke) => {
            tracing::trace!(name="ingame-input-smoke", region=?region);
            commands.trigger(UpdateSmoke(*region, smoke.clone()));
        }
        ToClient::Fx(fx) => {
            tracing::trace!(name="ingame-input-fx", fx=?fx);
            commands.trigger(FxEvent(fx.clone()));
        }
        ToClient::UpdateVisibilities(visibilities, vehicles) => {
            tracing::trace!(name = "ingame-input-visibilities");
            commands.trigger(UpdateVisibilities(visibilities.clone(), vehicles.clone()));
        }
        ToClient::SetBattlePhase(phase) => {
            tracing::trace!(name = "ingame-input-set-battle-phase");
            commands.trigger(SetBattlePhase::new(*phase));
        }
        ToClient::Elapsed(elapsed, duration) => {
            commands.trigger(UpdateRemaining(duration.saturating_sub(*elapsed)))
        }
        ToClient::Flags(flags) => commands.trigger(UpdateFlags(flags.clone())),
        ToClient::Morale(morale) => commands.trigger(UpdateMorale(*morale)),
        ToClient::SetSupportCount(type_, count) => {
            commands.trigger(SetSupportCountEvent(type_.clone(), *count))
        }
        ToClient::SupportExecuted(type_) => commands.trigger(SupportExecutedEvent(type_.clone())),
        ToClient::Collision(position, lag) => commands.trigger(CollisionEvent(*position, *lag)),
    }
}
