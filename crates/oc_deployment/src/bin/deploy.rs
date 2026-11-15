use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use oc_deployment::generator::{Generator, profile::Profile};
use oc_mod::Mod;
use oc_root::format::read_yaml;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    profile: PathBuf,

    #[arg(long, short)]
    r#mod: PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let mod_ = Mod::load(&args.r#mod, None);
    let mod_ = mod_.context(format!("Load mod from {}", args.r#mod.display()))?;

    let profile: Profile = read_yaml(&args.profile)?;

    let deployment = Generator::new()
        .generate(&mod_, &profile)
        .context("Generate deployment")?;
    let yaml = serde_yaml::to_string(&deployment).context("Serialize deployment")?;
    println!("{}", yaml);

    Ok(())
}
