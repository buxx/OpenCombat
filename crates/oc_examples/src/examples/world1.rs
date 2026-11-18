#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use crate::{logging, run, snapshot::SnapshotBuilder};
use anyhow::Context;
use oc_mod::Mod;
use oc_root::{WorldConfig, battle::BattlePhase, physics::Meters};
use oc_world::meta::Meta;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let mod_ = Mod::load(&PathBuf::from("mods/std1"), None)?;
    let meta = Meta::from_file(&PathBuf::from("maps/world1/meta.toml"))?;
    let map_ = PathBuf::from("maps/world1");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    let snapshot = SnapshotBuilder::new(
        map,
        vec![],
        vec![],
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .build(w, &mod_)?;

    let example = run::Example::builder()
        .world(PathBuf::from("maps/world1"))
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot);
    example.build().run()?;

    Ok(())
}
