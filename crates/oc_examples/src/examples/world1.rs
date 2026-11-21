use std::path::PathBuf;

use crate::{logging, run, snapshot::SnapshotBuilder, utils::Loaded};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let mod_ = PathBuf::from("mods/std1");
    let map = PathBuf::from("maps/world1");
    let Loaded {
        mod_: mod__,
        map: map_,
        w,
        ..
    } = Loaded::new(&mod_, &map)?;
    let snapshot = SnapshotBuilder::fight(map_, vec![], vec![]).build(w, &mod__)?;

    run::Example::builder()
        .world(map)
        .mod_(mod_)
        .snapshot(snapshot)
        .build()
        .run()?;

    Ok(())
}
