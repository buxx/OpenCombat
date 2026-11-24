use oc_root::format::RkyvFile;
use std::path::{Path, PathBuf};

use anyhow::Context;
use oc_root::uuids::SquadUuid;
use oc_root::{files::Roots, side::Side, static_::StaticSource};
use oc_world::{
    load::WorldPath,
    meta::Meta,
    reader::MapReader,
    support::{Support, SupportType, Supports},
};

use crate::{
    config::ServerConfig,
    end::EndPolicy,
    generate::{Generate, InitialOrder, SnapshotSource},
};

/// File name (under derived files directory) of generated snapshot
pub const GENERATED_SNAPSHOT_FILE: &str = "snapshot";

#[derive(clap::Args, Debug, Clone)]
pub struct ServerArgs {
    /// Mod directory (like ./mods/std1)
    pub mod_: PathBuf,
    /// World directory (like ./maps/world1)
    pub world: PathBuf,
    #[command(flatten)]
    pub snapshot: SnapshotArgs,
    /// Region width (default: map width)
    #[clap(long)]
    pub region_width: Option<u64>,
    /// Region height (default: map height)
    #[clap(long)]
    pub region_height: Option<u64>,
    /// Directory of derived files (generated snapshot is written there)
    #[clap(long, default_value = ".cache")]
    pub cache: PathBuf,
    /// What to do when battle ends: `none` or `save-map-persist-to:<path>`
    #[clap(long, default_value = "none")]
    pub end: EndPolicy,
    /// Side played by the computer (AI), like `b`
    #[clap(long, value_enum)]
    pub ai: Option<Side>,
    /// Behavior of the AI (see `--ai`)
    #[clap(long, value_enum, default_value_t)]
    pub ai_kind: oc_ai::AiKind,
}

impl ServerArgs {
    pub fn ai(&self) -> Option<crate::ai::AiConfig> {
        self.ai.map(|side| crate::ai::AiConfig {
            side,
            kind: self.ai_kind,
            placed: self.snapshot.places.iter().map(|(uuid, _)| *uuid).collect(),
        })
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct SnapshotArgs {
    /// Snapshot file (initial battle state). If not given, it is built from deployments,
    /// spawns, places, supports and orders arguments.
    #[clap(long)]
    pub snapshot: Option<PathBuf>,
    /// Deployment file (yaml) of side A
    #[clap(
        long,
        required_unless_present = "snapshot",
        conflicts_with = "snapshot"
    )]
    pub deployment_a: Option<PathBuf>,
    /// Deployment file (yaml) of side B
    #[clap(
        long,
        required_unless_present = "snapshot",
        conflicts_with = "snapshot"
    )]
    pub deployment_b: Option<PathBuf>,
    /// Map spawn zones owned by side A at start, like `SouthEast,South`
    #[clap(
        long,
        value_delimiter = ',',
        required_unless_present = "snapshot",
        conflicts_with = "snapshot"
    )]
    pub spawns_a: Vec<String>,
    /// Map spawn zones owned by side B at start, like `North,West`
    #[clap(
        long,
        value_delimiter = ',',
        required_unless_present = "snapshot",
        conflicts_with = "snapshot"
    )]
    pub spawns_b: Vec<String>,
    /// Place a squad at a map place: `<squad uuid>=<place name>` (repeatable).
    #[clap(long = "place", value_parser = parse_place, conflicts_with = "snapshot")]
    pub places: Vec<(SquadUuid, String)>,
    /// Side A available support: `<type>=<count>`, like `"Naval bombard=3"` (repeatable). Type is
    /// a support name declared in mod `support.ron`.
    #[clap(long = "support-a", value_parser = parse_support, conflicts_with = "snapshot")]
    pub supports_a: Vec<(SupportType, u8)>,
    /// Side B available support: `<type>=<count>`, like `"Naval bombard=3"` (repeatable). Type is
    /// a support name declared in mod `support.ron`.
    #[clap(long = "support-b", value_parser = parse_support, conflicts_with = "snapshot")]
    pub supports_b: Vec<(SupportType, u8)>,
    /// Initial order of side A squads (default: hide, or defend if client plays side B)
    #[clap(long, value_enum, conflicts_with = "snapshot")]
    pub order_a: Option<InitialOrder>,
    /// Initial order of side B squads (default: hide, or defend if client plays side A)
    #[clap(long, value_enum, conflicts_with = "snapshot")]
    pub order_b: Option<InitialOrder>,
    /// World state file (`.ocws`) saved by a previous battle on this map: its altered tiles,
    /// decor, craters and wrecks are used instead of map original ones.
    #[clap(long, conflicts_with = "snapshot")]
    pub persisted: Option<PathBuf>,
}

impl ServerArgs {
    /// Files roots: sources are used in place, derived files go under cache
    pub fn roots(&self) -> anyhow::Result<Roots> {
        let parent = |path: &Path| -> anyhow::Result<PathBuf> {
            Ok(std::path::absolute(path)
                .context(format!("Absolute path of {}", path.display()))?
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default())
        };
        Roots::in_place(&parent(&self.mod_)?, &parent(&self.world)?, &self.cache)
            .context("Build files roots")
    }

    /// Prepare server files (including generated snapshot if any) and build server config.
    /// `player` is the side of the client embedding the server, if any (used for default
    /// generated snapshot orders).
    pub fn server_config(
        &self,
        static_: StaticSource,
        player: Option<Side>,
    ) -> anyhow::Result<(ServerConfig, MapReader)> {
        let roots = self.roots()?;
        roots
            .ensure_server_dirs()
            .context(format!("Create dirs under {}", roots.derived.display()))?;

        let meta = self.world.meta();
        Meta::from_file(&meta).context(format!("Read file {}", meta.display()))?;
        let map =
            MapReader::new(&self.world).context(format!("Read map {}", self.world.display()))?;
        let map_width = map.width().context("Read map width")?;
        let map_height = map.height().context("Read map height")?;

        let snapshot = match self.snapshot.source()? {
            SnapshotSource::File(path) => path,
            SnapshotSource::Generate(generate) => {
                let path = roots.derived.join(GENERATED_SNAPSHOT_FILE);
                tracing::info!("Generate snapshot ({})", path.display());
                generate
                    .snapshot(&self.mod_, &self.world, &map, player)
                    .context("Generate snapshot")?
                    .save(&path)
                    .context(format!("Save snapshot to {}", path.display()))?;
                path
            }
        };

        let config = ServerConfig::builder()
            .world(self.world.clone())
            .mod_(self.mod_.clone())
            .roots(roots)
            .static_(static_)
            .snapshot(snapshot)
            .region_width(self.region_width.unwrap_or(map_width as u64))
            .region_height(self.region_height.unwrap_or(map_height as u64))
            .build();

        Ok((config, map))
    }
}

impl SnapshotArgs {
    pub fn source(&self) -> anyhow::Result<SnapshotSource> {
        if let Some(path) = &self.snapshot {
            return Ok(SnapshotSource::File(path.clone()));
        }

        // Presence ensured by clap `required_unless_present`
        let (Some(deployment_a), Some(deployment_b)) = (&self.deployment_a, &self.deployment_b)
        else {
            anyhow::bail!("--deployment-a and --deployment-b are required without --snapshot");
        };

        Ok(SnapshotSource::Generate(
            Generate::builder()
                .deployment_a(deployment_a.clone())
                .deployment_b(deployment_b.clone())
                .spawns_a(self.spawns_a.clone())
                .spawns_b(self.spawns_b.clone())
                .places(self.places.clone())
                .supports(Supports::new(
                    Support(self.supports_a.clone()),
                    Support(self.supports_b.clone()),
                ))
                .maybe_order_a(self.order_a)
                .maybe_order_b(self.order_b)
                .maybe_persisted(self.persisted.clone())
                .build(),
        ))
    }
}

fn parse_place(value: &str) -> Result<(SquadUuid, String), String> {
    let (uuid, name) = value
        .split_once('=')
        .ok_or_else(|| format!("Expected `<squad uuid>=<place name>`, got '{value}'"))?;
    let uuid = uuid
        .parse::<SquadUuid>()
        .map_err(|e| format!("Invalid squad uuid '{uuid}': {e}"))?;

    Ok((uuid, name.to_string()))
}

fn parse_support(value: &str) -> Result<(SupportType, u8), String> {
    let (type_, count) = value
        .rsplit_once('=')
        .ok_or_else(|| format!("Expected `<type>=<count>`, got '{value}'"))?;
    if type_.is_empty() {
        return Err(format!("Empty support type in '{value}'"));
    }
    let type_ = SupportType::new(type_);
    let count = count
        .parse()
        .map_err(|e| format!("Invalid support count '{count}': {e}"))?;

    Ok((type_, count))
}
