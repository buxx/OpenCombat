use std::path::{Path, PathBuf};

use anyhow::Context;
use bon::Builder;
use oc_deployment::deployment::{
    Deployments,
    place::{OrderPolicy, Placer},
};
use oc_mod::Mod;
use oc_root::uuids::SquadUuid;
use oc_root::{
    WorldConfig,
    battle::BattlePhase,
    geo::WorldVec2,
    physics::Meters,
    side::{BySide, Side},
};
use oc_world::{
    load::WorldPath, meta::Meta, reader::MapReader, snapshot::Snapshot, spawn::SpawnZone,
    support::Supports,
};

#[derive(Debug, Clone)]
pub enum SnapshotSource {
    /// Use an existing snapshot file
    File(PathBuf),
    /// Build snapshot from world, deployments, placements and supports
    Generate(Generate),
}

/// Initial order given to squads of a side
#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum InitialOrder {
    None,
    Hide,
    Defend,
}

impl From<InitialOrder> for OrderPolicy {
    fn from(value: InitialOrder) -> Self {
        match value {
            InitialOrder::None => OrderPolicy::None,
            InitialOrder::Hide => OrderPolicy::Hide,
            InitialOrder::Defend => OrderPolicy::Defend,
        }
    }
}

#[derive(Debug, Clone, Builder)]
pub struct Generate {
    /// Deployment file of side A
    deployment_a: PathBuf,
    /// Deployment file of side B
    deployment_b: PathBuf,
    /// Map spawn zone names owned by side A at start
    spawns_a: Vec<String>,
    /// Map spawn zone names owned by side B at start
    spawns_b: Vec<String>,
    /// Squad (by deployment uuid) to place at map place (by name).
    #[builder(default)]
    places: Vec<(SquadUuid, String)>,
    #[builder(default)]
    supports: Supports,
    /// Initial order of side A squads (default: see [`Generate::snapshot`])
    order_a: Option<InitialOrder>,
    /// Initial order of side B squads (default: see [`Generate::snapshot`])
    order_b: Option<InitialOrder>,
}

impl Generate {
    /// Build snapshot in deployment phase. Without explicit initial orders: when `player` side is
    /// known (server embedded in a client) its squads hide and others defend, elsewhere all hide.
    pub fn snapshot(
        &self,
        mod_: &Path,
        world: &Path,
        map: &MapReader,
        player: Option<Side>,
    ) -> anyhow::Result<Snapshot> {
        let mod_ =
            Mod::load(&mod_.to_path_buf(), None).context(format!("Load mod {}", mod_.display()))?;
        let meta_path = world.to_path_buf().meta();
        let meta =
            Meta::from_file(&meta_path).context(format!("Read file {}", meta_path.display()))?;
        let w = WorldConfig::new(
            map.width().context("Read map width")? as u64,
            map.height().context("Read map height")? as u64,
        )
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));

        let zones = map.spawn_zones().context("Read map spawn zones")?;
        let spawns = |names: &[String], side: Side| -> anyhow::Result<Vec<SpawnZone>> {
            if names.is_empty() {
                anyhow::bail!("Side {side:?} must own at least one spawn zone");
            }
            names
                .iter()
                .map(|name| {
                    zones
                        .iter()
                        .find(|z| &z.name.0 == name)
                        .cloned()
                        .context(format!("Spawn zone '{name}' not found in map"))
                })
                .collect()
        };
        let (spawns_a, spawns_b) = (
            spawns(&self.spawns_a, Side::A)?,
            spawns(&self.spawns_b, Side::B)?,
        );

        let deployments = Deployments::from_files(&self.deployment_a, &self.deployment_b)?
            .with_spawns(spawns_a, spawns_b);

        let map_ = map.build().context("Build map")?;
        let places = map_.places();
        let place = |name: &str| -> anyhow::Result<WorldVec2> {
            places
                .iter()
                .find(|p| p.name.0 == name)
                .map(|p| WorldVec2::new(p.x, p.y))
                .context(format!("Place '{name}' not found in map"))
        };
        let assigned = self
            .places
            .iter()
            .map(|(uuid, name)| Ok((*uuid, place(name)?)))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let placer = Placer::new(places.iter().map(|p| WorldVec2::new(p.x, p.y)).collect())
            .with_assigned(assigned);
        let default = |side: Side| match player {
            Some(player) if player != side => InitialOrder::Defend,
            _ => InitialOrder::Hide,
        };
        let orders = BySide::new(self.order_a, self.order_b);
        let placer = placer.with_orders(BySide::from_fn(|side| {
            orders[side].unwrap_or_else(|| default(side)).into()
        }));

        for (type_, _) in [Side::A, Side::B]
            .into_iter()
            .flat_map(|side| self.supports[side].iter())
        {
            if mod_.support(type_.name()).is_none() {
                let known = mod_.supports.iter().map(|s| s.name.as_str());
                anyhow::bail!(
                    "Support '{type_}' not declared in mod (known: {})",
                    known.collect::<Vec<_>>().join(", ")
                );
            }
        }

        let tiles = map.tiles(&w, &mod_).context("Read map tiles")?;
        let mut snapshot = Snapshot::empty(w.clone())
            .with_tiles(tiles.clone())
            .with_decor(map.decor().context("Read map decor")?)
            .with_supports(self.supports.clone())
            .with_phase(BattlePhase::Deployment);
        snapshot.spawns = deployments
            .as_ref()
            .map(|d| d.spawns.iter().map(|s| s.name.clone()).collect());

        deployments
            .mobilize(&w, &mod_, &mut snapshot, &placer, &tiles)
            .context("Mobilize deployments")?;

        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use clap::ValueEnum;

    use super::*;

    #[test]
    fn order_policy_arg_name_is_initial_order_value() {
        for policy in [OrderPolicy::None, OrderPolicy::Hide, OrderPolicy::Defend] {
            let order = InitialOrder::from_str(policy.arg_name(), false).unwrap();
            assert_eq!(OrderPolicy::from(order), policy);
        }
    }
}
